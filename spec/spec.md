# dsc — specification

This file contains normative design standards and internal specifications for `dsc`. Per-command documentation lives in [docs/](../docs/).

## Global options

- `dsc --config <path> <command>` (or `-c <path>`) to select a config file. Without `--config`, `dsc` searches common local/user/system paths and falls back to `./dsc.toml`.
- CLI help quality requirement: every command, subcommand, argument, and flag must include concise `--help` text.

## CLI consistency standards

### Source of truth and docs policy

- `dsc --help` is the source of truth for complete command/flag coverage.
- README should showcase features and installation, then index into `docs/` for details.

### Format baseline

- All `* list` commands must support `--format text|json|yaml` at minimum.
- `text` remains the default format.
- Commands that already support richer formats (`markdown`, `markdown-table`, `csv`, `urls`) may keep those extras.
- Where practical, format rendering should reuse shared helpers and avoid duplicated formatter logic.

### Error message standards

- Missing discourse config entry:
  - `discourse not found: {name}`
- Missing remote resource:
  - `{resource} not found: {identifier}`
- Missing required config values:
  - `missing {field} for {resource}; please set {field} or check your config`
- Input validation should fail fast before API calls where possible (for example, empty identifiers).

### Empty list behavior

- For `--format text`, list commands should print:
  - `No <resource> found.`
- For structured formats (`json|yaml`), empty collections should serialize as normal empty arrays/objects.

### Success output policy

- Mutating commands should be pipe-friendly by default.
- Prefer machine-usable output in `text` mode (for example resource URLs/paths/IDs in stable single-line formats) over prose-only sentences.
- Long-term direction:
  - add shared `--quiet` and `--format json` behavior for mutating commands.

### Cross-target semantics

- Equivalent cross-instance operations should share flag semantics.
- `category copy` and `group copy` should both support `--target`.

### Flag style

- Short flags should be lowercase.
- Use `--parallel` / `-p` for concurrency semantics.

### Conflict resolution for review notes

- Implementation baseline is:
  - minimum `text|json|yaml` for every list command
  - preserve richer formats where already present
  - expand beyond baseline incrementally when justified.

## Internals

### dsc.toml spec

See [docs/configuration.md](../docs/configuration.md) for the user-facing reference. The canonical field list and placeholder semantics are defined there.

### Release / distribution

- GitHub Releases will ship prebuilt binaries for:
  - `x86_64-unknown-linux-gnu`
  - `aarch64-unknown-linux-gnu`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- crates.io publishing is automated in CI on `v*` tags (requires `CARGO_REGISTRY_TOKEN`).
- `CHANGELOG.md` should be updated for each release.
- Team workflow: commit regularly during active work.
- Team workflow: bump the crate version at least once per day when working on `dsc` (use `s/version++`).
