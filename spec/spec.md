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

## Command patterns

These are the load-bearing patterns `dsc` has converged on. They are normative for new commands. The general rationale (and the same patterns for any CLI) lives in `~/code/house-style/rust-cli.md`; this section records how `dsc` applies them.

### Declarative resource sync: `pull` → edit → `push` (→ `diff`)

Anything that manages stateful server resources follows the sync loop, not one-off setters. Exemplars: `setting pull/push/diff`, `tag pull/push`, `category pull/push`, `theme setting pull/push`, `theme field pull/push`.

- `pull` writes a local snapshot: YAML by default, JSON when the path ends `.json`. The file carries a `version: 1` schema integer and provenance (`pulled_at`, and the Discourse version where known). `push` rejects an unknown schema version.
- Server encodings that are awkward to hand-edit are expanded on pull and re-collapsed on push. The reference case: theme JSON-schema list settings (`header_links`) arrive as a string of escaped JSON and are expanded to a real YAML list; `theme field` bodies (SCSS/HTML) are written as raw text files.
- `push` is idempotent: read current server state, send **only the entries that differ**, and report the plan (changed / unchanged / unknown-on-server / reset). An untouched `pull → push` is a verified no-op.
- **Compare parsed values, never raw strings.** The file's serialisation differs from the server's (compact vs spaced, key order); string comparison makes everything look changed. This bit us on `theme setting push` - the fix was semantic (parsed-JSON) comparison. New sync commands must do the same.

### `--dry-run` / `-n`

Global flag. Every mutating command honours it and prints the **complete** plan it would execute (resolved names, the diff, the requests) while touching nothing - it is the review gate before acting on a production forum. Read-only commands accept and ignore it so scripts can pass it uniformly. Provisioning flows (`backup setup-s3`) must print the whole plan offline under `--dry-run`, including the exact external commands.

### Guard destructive / irreversible actions

- Refuse the dangerous case with a message that names the safe alternative: `theme delete` refuses the site default; `theme field push` refuses a git-backed remote and points at `theme update` / `theme duplicate`.
- Never print secrets: redact URL-embedded credentials (`https://***@host`) in output; an API secret is shown only on the one creation line.
- Sequence multi-step provisioning "enable last" (set up dependencies, flip the activating switch at the end) so a mid-run failure leaves a clean disabled state - as used for `backup setup-s3` (`backup_location=s3` last) and reply-by-email.

### Every client capability has a command

If `src/api/*` exposes a method that performs a distinct Discourse operation, a command reaches it. `delete_theme` once existed with no command wiring it - a dead capability. `pub fn` suppresses the dead-code lint, so audit periodically: each public client method should be referenced from `src/commands`/`main.rs` directly or transitively.

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
