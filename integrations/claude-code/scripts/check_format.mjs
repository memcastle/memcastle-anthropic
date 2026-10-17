// Shipped assets and tests should not vary with editor-specific whitespace settings.
import { readFileSync } from "node:fs"
import { dirname, join } from "node:path"
import { fileURLToPath } from "node:url"

const root = dirname(dirname(fileURLToPath(import.meta.url)))
const files = ["plugin/.mcp.json", "plugin/.claude-plugin/marketplace.json"]
for (const file of files) {
  const data = readFileSync(join(root, file), "utf8")
  if (data.includes("\r") || !data.endsWith("\n") || data.split("\n").some((line) => /[\t ]+$/.test(line))) {
    throw new Error(`${file}: use LF, remove trailing whitespace and add a final newline`)
  }
}
