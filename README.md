# MemCastle Anthropic plugin

Coverage CI measures the native Rust source tests and Claude Code Bun integration tests, then uploads LCOV
to Codecov with GitHub OIDC. Enable this repository in Codecov for PR coverage status.

Claude Code history and lifecycle integration.

This independently released plugin contains the existing `claude` Claude Code transcript source and the `claude-code`
Claude Code agent integration. It does not currently include a Claude.ai conversation-history source.
The stable module IDs preserve miner configuration, mined source identity and existing Claude Code installation receipts.
This provider plugin owns the versioned package, while each module has its own lifecycle.
Installing the plugin makes its modules available; enable sources and install integrations separately.

```sh
python3 scripts/package.py                         # requires Rust and wasm32-wasip2
python3 scripts/check_modules.py
python3 -m unittest discover -s tests
MEMCASTLE_BIN=/path/to/memcastle python3 scripts/smoke_install.py
MEMCASTLE_BIN=/path/to/memcastle python3 scripts/smoke_adoption.py
```

The archive in `dist/` is independently releasable from MemCastle Core.
The root `plugin.toml` declares stable module identities; the package script fills artifact hashes after building.
Commit each source module's `Cargo.lock` and regenerated `plugin.toml` hashes before publishing a release.
The source module carries the versioned public WIT contract and its own fixtures; the agent integration uses MCP/HTTP.
No module needs a private Core checkout or a sibling provider to build or test.
The smoke check starts an isolated daemon, upgrades an existing Core-shipped Claude Code installation receipt, and
verifies registry discovery, source mining and the agent integration in a temporary home.
The adoption check separately proves installed and formerly bundled `claude` miners retain their cursors and documents.
Provider CI builds and packages locally, then calls Core's shared artifact-validation workflow for a `[main, versions...]`
matrix, with additional exact plugin-capable
stable versions configured in `core-versions` and the `main` prerelease always included.
The reusable validation workflow follows Core's `main` branch without a revision pin.
It runs on every pull request and every push to `main`, including merges, without a second push run on PR branches.
Every version exercises source fixtures and archive installation/activation; incompatible versions fail the workflow.
The development release body records the exact Core source SHA, and the workflow summary reports it for each run.
Core CI separately tests a freshly composed project against its current install contract.
The provider's own release workflow accepts only a `anthropic/v<plugin.toml version>` tag on `main`, then builds
the reviewed artifact, runs manifest and compatibility-matrix tests, and refuses an existing release.
## Install and migrate

Start the MemCastle daemon, then run `memcastle plugin search anthropic` and `memcastle plugin install anthropic`.
The plugin installs `claude` inactive; enable it with `memcastle source enable claude` when you are ready to mine.
Install the agent side separately with `memcastle integration install claude-code` (this does not require a daemon).
On a previously mined palace, remove the old `claude` package record with `memcastle source remove claude` after checking
that the old Core bundle is no longer present, then preview the plugin and install with `--adopt-source claude` and the
consent digest it requests. The source identity, `dir` scope, cursor and mined drawers stay in the palace;
the old source's enabled state must be restored explicitly with `memcastle source enable claude`.
An existing Claude Code agent installation can be refreshed with `memcastle integration update claude-code` after the
plugin has been installed; the local marketplace name `memcastle-local` and plugin name `memcastle` do not change.

## Contribute and release

Develop the modules under `modules/claude/` and `modules/claude-code/` and run the commands above before review.
The recorded GitTPL parent and child revisions live in `.config/git.tpl.toml`; template updates must be reviewed and
merged explicitly. CI checks both module projects and tests the packaged artifact against Core's plugin-capable `main`
release, including legacy source adoption. Do not list Core 0.4.1 as compatible: it predates plugin installation.
To publish, review the committed artifact digests and CI result, then tag a reviewed `main` commit
`anthropic/v<plugin.toml version>`; the standalone release workflow publishes the validated archive.
Core's `plugins.json` catalogue discovers that release by repository and checks its release-asset digest.
