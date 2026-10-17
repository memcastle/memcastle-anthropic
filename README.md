# MemCastle Anthropic plugin

Claude Code history and lifecycle integration.

This independently released plugin contains the existing `claude` Claude Code transcript source and the `claude-code`
Claude Code agent integration. It does not currently include a Claude.ai conversation-history source.
The stable module IDs preserve miner configuration, mined source identity and existing Claude Code installation receipts.
This provider plugin owns the versioned package, while each module has its own lifecycle.
Installing the plugin makes its modules available; enable sources and install integrations separately.

```sh
memcastle plugin inspect plugin.toml --json
# Build sources/claude for wasm32-wasip2 and run bun run build in integrations/claude-code first.
memcastle plugin package plugin.toml
memcastle plugin test dist/anthropic-0.1.0.tar.gz
MEMCASTLE_BIN=/path/to/memcastle node scripts/smoke_adoption.mjs
```

The archive in `dist/` is independently releasable from MemCastle Core.
The root `plugin.toml` is an immutable module descriptor; Core regenerates the reviewable `manifest.toml` with
release versions and artifact checksums. Commit each source module's `Cargo.lock` and regenerated `manifest.toml`.
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
The release workflow validates the reviewed `anthropic/v<manifest.toml version>` tag on `main`, then builds and
checks the archive against Core's compatibility matrix before attaching it to gh-ship's draft release.
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

Develop the modules under `sources/claude/` and `integrations/claude-code/` and run the commands above before review.
The recorded GitTPL parent and child revisions live in `.config/git.tpl.toml`; template updates must be reviewed and
merged explicitly. CI checks both module projects and tests the packaged artifact against Core's plugin-capable `main`
release, including legacy source adoption. Do not list Core 0.4.1 as compatible: it predates plugin installation.
Merging a releasable Conventional Commit into `main` opens a reviewed `release/next` PR via git-cliff and gh-ship.
After that PR merges, the release workflow publishes the validated archive and `.sha256sum`.
Configure the `release` environment with `APP_CLIENT_ID` and `APP_PRIVATE_KEY` for an installed GitHub App.
Core's `plugins.json` catalogue discovers that release by repository and checks its release-asset digest.
