# Roadmap

Tracks planned, in-progress, and completed work items for `dsc`.

## Completed

- [x] `dsc tag pull/push` - declarative tag taxonomy management ([spec](spec.md) via `dsc-tag-sync-spec.md`)
- [x] `dsc topic tag/untag` - moved topic-level tagging from `dsc tag apply/remove`
- [x] `dsc post pull/push` - harmonised with pull/push pattern
- [x] `dsc backup pull/push` - harmonised with pull/push pattern
- [x] `dsc emoji pull/push` - harmonised with pull/push pattern
- [x] `dsc config` (bare) - print active config path and search order
- [x] `dsc update` rootless Docker support - `docker_rootless` config flag
- [x] `dsc update` skip-if-current - check GitHub for latest stable commit before rebuild
- [x] `dsc harden` PQ-hybrid SSH - policy overlay approach for KEX/ciphers/MACs
- [x] Pin GitHub Actions to commit SHAs
- [x] Dependabot cooldown configuration

## In progress

_(nothing currently in progress)_

## Planned

### Setting sync (bulk site settings pull/push)

Spec: [spec/setting-sync.md](setting-sync.md)

Snapshot and version-control Discourse site settings. Enables staging→production workflows.

- [ ] **Phase 1: `dsc setting pull`** - snapshot all settings (with metadata) to YAML/JSON. `--changed-only` for manageable diffs. Self-documenting file format with `default`, `description`, `type` per entry.
- [ ] **Phase 2: `dsc setting push`** - idempotent reconciliation. Only PUTs changed values. `--reset-unlisted` for full sync. `--dry-run` shows diff plan.
- [ ] **Phase 3: `dsc setting diff`** - compare two instances or two snapshot files side-by-side.
- [ ] **Phase 4: Fix `setting set --tags`** - make `discourse` optional when `--tags` is provided (code exists, CLI blocks it).

### Other planned items

- [ ] `dsc tag rename <old> <new>` - rename a tag preserving topic associations (avoids the delete+create problem in pull/push)
- [ ] Shell completion regeneration - refresh `completions/` for new subcommands (`topic tag/untag`, `tag pull/push`, `setting pull/push`)
- [ ] Fix `dsc harden` test - `drop_in_uses_modern_algorithm_pins` test failure from policy overlay changes
- [ ] Doc accuracy pass - `setting get --format` is documented but not implemented; verify all docs match CLI reality
