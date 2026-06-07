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
- [x] **Setting sync (bulk site settings pull/push)** - declarative settings management. Spec: [spec/setting-sync.md](setting-sync.md)
  - [x] Phase 1: `dsc setting pull` - snapshot to YAML/JSON with metadata
  - [x] Phase 2: `dsc setting push` - idempotent apply with `--dry-run` and `--reset-unlisted`
  - [x] Phase 3: `dsc setting diff` - cross-source comparison (live or snapshot)
  - [x] Phase 4: `setting set --tags` reachable from CLI

## In progress

_(nothing currently in progress)_

## Planned

### Other planned items

- [ ] `dsc tag rename <old> <new>` - rename a tag preserving topic associations (avoids the delete+create problem in pull/push)
- [ ] Shell completion regeneration - refresh `completions/` for new subcommands (`topic tag/untag`, `tag pull/push`, `setting pull/push`)
- [ ] Fix `dsc harden` test - `drop_in_uses_modern_algorithm_pins` test failure from policy overlay changes
- [ ] Doc accuracy pass - verify remaining docs match CLI reality
