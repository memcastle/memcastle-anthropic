// Both installed and formerly bundled Claude history survive plugin ownership.
import assert from "node:assert/strict"
import { copyFileSync, mkdirSync, readFileSync, rmSync, utimesSync } from "node:fs"
import { join } from "node:path"
import { ROOT, cli, isolated, mine } from "./smoke_common.mjs"

cli(["source", "package", join(ROOT, "modules/claude"), "--no-build"], process.env)
const plugin = readFileSync(join(ROOT, "dist/anthropic-0.1.0.tar.gz"))
const legacy = readFileSync(join(ROOT, "modules/claude/dist/claude-0.1.0.tar.gz"))

for (const bundled of [false, true]) {
  await isolated(`memcastle-claude-${bundled ? "bundled" : "installed"}-adopt-`, ({ assets }) => {
    if (!bundled) return
    const source = join(assets, "sources/claude")
    mkdirSync(source, { recursive: true })
    copyFileSync(join(ROOT, "modules/claude/memcastle-source.toml"), join(source, "memcastle-source.toml"))
    copyFileSync(join(ROOT, "modules/claude/dist/source.wasm"), join(source, "source.wasm"))
  }, async ({ root, assets, request }) => {
    if (bundled) {
      await request("/api/source-packages/claude/enable", { method: "POST" })
    } else {
      let digest
      try { await request("/api/source-packages", { method: "POST", data: legacy }) }
      catch (error) {
        assert.equal(error.body.code, "memcastle::source::consent_required")
        digest = error.body.help.split("--consent ")[1].split("`")[0]
      }
      assert.ok(digest, "legacy source must request permission consent")
      await request(`/api/source-packages?consent=${encodeURIComponent(digest)}&enable=true`, { method: "POST", data: legacy })
    }
    const sessions = join(root, "sessions")
    const session = join(sessions, "-work-app/session-a.jsonl")
    mkdirSync(join(sessions, "-work-app"), { recursive: true })
    copyFileSync(join(ROOT, "modules/claude/fixtures/claude-projects/tree/-work-app/session-a.jsonl"), session)
    utimesSync(session, 1000, 1000)
    await request("/api/miners/claude", { method: "PUT", json: { source: "claude", locator: sessions } })
    const run = async () => {
      const job = await mine(request, "claude", sessions)
      assert.equal(job.status, "completed", JSON.stringify(job))
      return job.result
    }
    assert.equal((await run()).created, 1)
    const before = (await request("/api/sources")).sources.find((item) => item.source === "claude")
    if (bundled) rmSync(join(assets, "sources/claude"), { recursive: true })
    await request("/api/source-packages/claude", { method: "DELETE" })
    const preview = await request("/api/plugins/preview", { method: "POST", data: plugin })
    assert.deepEqual(preview.adoptions, ["claude"])
    const consents = Object.fromEntries(preview.source_consents.map(({ source, digest }) => [source, digest]))
    await assert.rejects(request("/api/plugins", { method: "POST", data: plugin,
      headers: { "X-MemCastle-Consents": JSON.stringify(consents) } }), { status: 409 })
    await request("/api/plugins", { method: "POST", data: plugin, headers: {
      "X-MemCastle-Consents": JSON.stringify(consents), "X-MemCastle-Adopt-Sources": '["claude"]',
    } })
    const after = (await request("/api/sources")).sources.find((item) => item.source === "claude")
    assert.deepEqual([after.cursor, after.documents], [before.cursor, before.documents])
    assert.equal((await request("/api/miners/claude")).source, "claude")
    assert.equal((await request("/api/source-packages/claude")).state, "installed")
    await request("/api/source-packages/claude/enable", { method: "POST" })
    assert.equal((await run()).documents, 0, "adoption rewound the Claude cursor")
  })
}
console.log("legacy installed and bundled Claude miners, cursors and mined history survived adoption")
