# Claude Code history source

## Build baseline

Requires Rust 1.100+, `wasm32-wasip3` / WASI 0.3, wit-bindgen 0.63, and source contract 0.5.0.
Use upstream development Core containing `828f0f6`; stable 0.5.2 predates this ABI.
Generate and commit `Cargo.lock`, then run `cargo build --locked --release --target wasm32-wasip3`.
The stackful `sync: true` bindings preserve synchronous Rust I/O behind async acquisition exports.
From the plugin root, `mise run source:claude:build` stages the component for packaging.
Run `python3 scripts/validate_component.py source sources/claude/source.wasm` with wasm-tools 1.263.0,
then `memcastle source test sources/claude` for the actual transcript fixture.

This WebAssembly source reads Claude Code's documented JSONL session transcripts from
`~/.claude/projects/<project>/<session>.jsonl`.
It is separate from any live Claude Code integration: it reads finished or active local transcript files directly and never
starts Claude Code, contacts a service, or reads Claude settings or credentials.

## Use it

```sh
memcastle plugin install anthropic
memcastle source enable claude
memcastle mine claude
memcastle mine claude /backups/claude/projects since=2026-09 dir=/work/app
```

It requests read-only access to the supplied locator and `~/.claude/projects`, plus `HOME` only to locate that default.
It has no process, network, write, or credential permission.

Each transcript is one `transcript` document tagged `claude` and `transcript`.
The source retains user and assistant text plus compact `[tool: name]` markers, and deliberately drops thinking, tool results,
system and compaction records, attachments, patches, malformed lines, and unknown records.
The source never follows symlinks and only considers `.jsonl` files exactly one directory below the projects root.

The `watch` capability is declarative only.
Define and enable a miner and trigger yourself; installing this package does not start background mining.
