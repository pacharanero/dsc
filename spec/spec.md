# dsc — specification

This file holds `dsc`'s **internal** specifications: the config schema and the release/distribution rules.

The **CLI design philosophy and command-surface standards** - output and formats, the `pull → edit → push → diff` sync loop, `--dry-run`, destructive-action guards, and the error/empty-list/flag conventions - now live in their own document: **[cli-design.md](cli-design.md)**. That is the normative reference for anything about how commands behave. Per-command usage lives in [docs/](../docs/); the cross-project CLI rules live in [`~/code/house-style/rust-cli.md`](../../../house-style/rust-cli.md).

## Internals

### dsc.toml spec

See [docs/configuration.md](../docs/configuration.md) for the user-facing reference. The canonical field list and placeholder semantics are defined there.

### Release / distribution

- GitHub Releases ship prebuilt binaries for:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- crates.io publishing is automated in CI on `v*` tags through crates.io Trusted Publishing, which exchanges the publish job's GitHub OIDC identity for a short-lived token.
- `CHANGELOG.md` is updated for each release (git-cliff).
- Team workflow: commit regularly during active work.
- Team workflow: bump the crate version at least once per day when working on `dsc` (use `s/version++`).
