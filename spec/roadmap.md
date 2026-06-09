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
- [x] **Config-path resolution** - `$DSC_CONFIG` and `$DSC_CONFIG_HOME` env vars, explicit-selector error semantics, `dsc config` source labelling. Spec: [spec/config-path-resolution.md](config-path-resolution.md)
- [x] `dsc tag rename <discourse> <old> <new>` - in-place rename preserving every topic association; pre-flight validates existence, name collisions, and slug shape
- [x] Fix `dsc harden` test - `drop_in_uses_modern_algorithm_pins` rewritten to verify the overlay model (commit `979c3d1`)

## In progress

_(nothing currently in progress)_

## Planned

### Other planned items

- [ ] Doc accuracy pass - verify remaining docs match CLI reality
- [ ] **Theme management gaps** - component settings, enable/disable + attach/detach, per-field editing, asset binding, `theme show`/`theme update`. Spec: [spec/theme-management.md](theme-management.md)
  - [ ] Phase 1: `dsc theme setting` (get/set/list) + `dsc theme enable|disable|attach|detach`
  - [ ] Phase 2: `dsc theme field pull/push` + `dsc theme asset set/list`
  - [ ] Phase 3: `dsc theme show` + `dsc theme update` (remote component refresh)

## Out of scope / removed

- ~~Shell completion regeneration~~ - `completions/` is gitignored and `dsc completions <shell> -d ./completions` regenerates on demand. Not a release-tracked item.
