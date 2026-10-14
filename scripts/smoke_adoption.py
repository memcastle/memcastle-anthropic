#!/usr/bin/env python3
"""Check both installed and formerly bundled Claude history survive plugin ownership."""

import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time
from urllib.error import HTTPError, URLError

from smoke_install import ROOT, request


def check_handoff(binary: Path, plugin: bytes, legacy: bytes, bundled: bool) -> None:
    # The second case reproduces a Core upgrade where the old bundled source is no longer shipped.
    kind = "bundled" if bundled else "installed"
    with tempfile.TemporaryDirectory(prefix=f"memcastle-claude-{kind}-adopt-") as scratch:
        root = Path(scratch)
        assets = root / "assets"
        assets.mkdir()
        if bundled:
            source = assets / "sources/claude"
            source.mkdir(parents=True)
            shutil.copyfile(ROOT / "modules/claude/memcastle-source.toml", source / "memcastle-source.toml")
            shutil.copyfile(ROOT / "modules/claude/dist/source.wasm", source / "source.wasm")
        env = {key: value for key, value in os.environ.items() if not key.startswith("MEMCASTLE_")}
        env.update({"HOME": scratch, "XDG_DATA_HOME": str(root / "data"),
                    "XDG_CONFIG_HOME": str(root / "config"), "XDG_STATE_HOME": str(root / "state"),
                    "MEMCASTLE_ASSETS_DIR": str(assets)})
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            port = probe.getsockname()[1]
        base = f"http://127.0.0.1:{port}"
        process = subprocess.Popen((str(binary), "--palace", str(root / "palace"), "serve",
                                    "--bind", "127.0.0.1", "--port", str(port)),
                                   cwd=root, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
        try:
            for _ in range(120):
                if process.poll() is not None:
                    raise RuntimeError(f"daemon exited: {process.stderr.read().decode()}")
                try:
                    request(base, "/api/health")
                    break
                except (URLError, TimeoutError):
                    time.sleep(0.25)
            else:
                raise AssertionError("disposable daemon did not become healthy")
            if bundled:
                request(base, "/api/source-packages/claude/enable", method="POST")
            else:
                install_legacy(base, legacy)
            sessions = root / "sessions" / "-work-app"
            sessions.mkdir(parents=True)
            session = sessions / "session-a.jsonl"
            shutil.copyfile(ROOT / "modules/claude/fixtures/claude-projects/tree/-work-app/session-a.jsonl", session)
            os.utime(session, (1000, 1000))
            request(base, "/api/miners/claude", method="PUT",
                    data=json.dumps({"source": "claude", "locator": str(sessions.parent)}).encode(),
                    headers={"Content-Type": "application/json"})

            def mine():
                job = request(base, "/api/jobs", method="POST", data=json.dumps({
                    "type": "mine", "source": "claude", "locator": str(sessions.parent), "requested_by": "test",
                }).encode(), headers={"Content-Type": "application/json"})
                for _ in range(300):
                    result = request(base, f"/api/jobs/{job['id']}")
                    if result["status"] in ("completed", "failed", "cancelled"):
                        assert result["status"] == "completed", result
                        return result["result"]
                    time.sleep(0.1)
                raise AssertionError("Claude mining did not complete within 30 seconds")

            assert mine()["created"] == 1
            before = next(item for item in request(base, "/api/sources")["sources"] if item["source"] == "claude")
            if bundled:
                shutil.rmtree(assets / "sources/claude")
            request(base, "/api/source-packages/claude", method="DELETE")
            preview = request(base, "/api/plugins/preview", method="POST", data=plugin)
            assert preview["adoptions"] == ["claude"], preview
            consents = {item["source"]: item["digest"] for item in preview["source_consents"]}
            try:
                request(base, "/api/plugins", method="POST", data=plugin,
                        headers={"X-MemCastle-Consents": json.dumps(consents)})
            except HTTPError as error:
                assert error.code == 409, error.read()
            else:
                raise AssertionError("legacy Claude history was adopted without explicit permission")
            request(base, "/api/plugins", method="POST", data=plugin, headers={
                "X-MemCastle-Consents": json.dumps(consents), "X-MemCastle-Adopt-Sources": '["claude"]',
            })
            after = next(item for item in request(base, "/api/sources")["sources"] if item["source"] == "claude")
            assert before["cursor"] == after["cursor"] and before["documents"] == after["documents"]
            assert request(base, "/api/miners/claude")["source"] == "claude"
            assert request(base, "/api/source-packages/claude")["state"] == "installed"
            request(base, "/api/source-packages/claude/enable", method="POST")
            assert mine()["documents"] == 0, "adoption must not rewind the source cursor"
        finally:
            process.terminate()
            try:
                process.communicate(timeout=20)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate()


def install_legacy(base: str, legacy: bytes) -> None:
    try:
        request(base, "/api/source-packages", method="POST", data=legacy)
    except HTTPError as error:
        refusal = json.load(error)
        assert refusal["code"] == "memcastle::source::consent_required", refusal
        digest = refusal["help"].split("--consent ", 1)[1].split("`", 1)[0]
    else:
        raise AssertionError("legacy Claude source installed without permissions consent")
    request(base, "/api/source-packages?consent=" + digest + "&enable=true", method="POST", data=legacy)


def main() -> None:
    binary = Path(os.environ["MEMCASTLE_BIN"]).resolve()
    plugin = (ROOT / "dist/anthropic-0.1.0.tar.gz").read_bytes()
    # The original source package and the plugin carry the same source ID and cursor encoding.
    subprocess.run((str(binary), "source", "package", str(ROOT / "modules/claude"), "--no-build"),
                   check=True, capture_output=True)
    legacy = (ROOT / "modules/claude/dist/claude-0.1.0.tar.gz").read_bytes()
    for bundled in (False, True):
        check_handoff(binary, plugin, legacy, bundled)
    print("legacy installed and bundled Claude miners, cursors and mined history survived adoption")


if __name__ == "__main__":
    main()
