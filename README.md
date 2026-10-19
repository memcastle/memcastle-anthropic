# MemCastle Anthropic plugin

[![CI][ci-badge]][ci-workflow] [![Codecov][codecov-badge]][codecov-project]

[ci-badge]: https://github.com/memcastle/memcastle-anthropic/actions/workflows/ci.yaml/badge.svg?branch=main
[ci-workflow]: https://github.com/memcastle/memcastle-anthropic/actions/workflows/ci.yaml
[codecov-badge]: https://codecov.io/gh/memcastle/memcastle-anthropic/graph/badge.svg
[codecov-project]: https://codecov.io/gh/memcastle/memcastle-anthropic

Claude Code history and lifecycle integration.

This independently released plugin contains the existing `claude` Claude Code transcript source and the
`claude-code` agent integration. It does **not** include a Claude.ai conversation-history source.
The stable module IDs preserve miner configuration, mined source identity and Claude Code installation receipts.

## Install and migrate

When a release is available in the [MemCastle plugin catalogue](https://memcastle.github.io/plugins.json),
start the MemCastle daemon and install by plugin ID with a plugin-capable Core binary:

```sh
memcastle plugin search anthropic
memcastle plugin install anthropic
memcastle source enable claude
memcastle integration install claude-code
```

Core retrieves and verifies the release archive. Installation leaves `claude` inactive until explicitly enabled;
installing the archive also does not register the agent integration. If there is no published release yet,
build the archive below and install `dist/anthropic-VERSION.tar.gz` instead.

On a previously mined palace, remove the old `claude` source-package record with
`memcastle source remove claude` after checking that the old Core bundle is no longer present.
Preview the plugin installation and use `--adopt-source claude` and the requested consent digest.
The source identity, `dir` scope, cursor and mined drawers stay in the palace, but the enabled state must be
restored with `memcastle source enable claude`.

An existing Claude Code agent installation can be refreshed with `memcastle integration update claude-code`
after installing the plugin; the local marketplace `memcastle-local` and plugin name `memcastle` do not change.

## Build and test locally

Install Rust with `wasm32-wasip2`, Bun and a plugin-capable MemCastle Core binary. Build the source component
to `sources/claude/source.wasm` and the integration into `integrations/claude-code/dist/` before packaging.
Commit the source `Cargo.lock` before releasing; see the CI workflow for the reproducible build commands.

```sh
memcastle plugin inspect --json
memcastle plugin package
memcastle plugin test dist/anthropic-VERSION.tar.gz
node scripts/smoke_adoption.mjs
```

Use the actual archive name in `dist/` instead of `VERSION`. Packaging leaves `plugin.toml` unchanged and puts
the versioned `manifest.toml` inside the archive, with an external `.sha256sum`. The installation smoke checks
registry discovery, source mining and agent integration in an isolated daemon and temporary agent home; it also
upgrades an existing Core-shipped Claude Code receipt. The adoption check ensures legacy `claude` miners retain
cursors and documents. Neither module needs a sibling Core checkout to build or test.

## CI and releases

CI builds both modules and tests the packaged artifact against Core `development`. The Core setup action verifies
the binary against `SHA256SUMS`; Core's shared validation tests source fixtures, installation and activation on
pull requests and pushes to `main`. Core 0.4.1 predates plugin installation and is not a compatible matrix entry.

gh-ship and git-cliff open a reviewed `release/next` PR for releasable commits. Merging it publishes the
validated `anthropic/vVERSION` archive. Configure the `release` environment with `APP_CLIENT_ID` and
`APP_PRIVATE_KEY` for an installed GitHub App. The plugin catalogue discovers published releases by repository
and checks their release-asset digest.

Codecov uses fixed 80% project and patch floors and an 81% `source-claude` component floor.
The `integration-claude-code` component remains informational until it uploads nonempty LCOV.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development and [AGENTS.md](AGENTS.md) for coding-agent guidance.
