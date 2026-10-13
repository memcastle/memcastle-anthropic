"""Keep shipped JavaScript and its tests free of whitespace-only changes."""

from pathlib import Path


root = Path(__file__).resolve().parents[1]
for path in (root / "dist/index.js", *sorted((root / "tests").glob("*.mjs"))):
    data = path.read_bytes()
    if b"\r" in data or not data.endswith(b"\n") or any(line.rstrip() != line for line in data.split(b"\n")):
        raise SystemExit(f"{path}: use LF, remove trailing whitespace and add a final newline")
