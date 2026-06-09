# Roadmap

Tracks planned, in-progress, and completed work items for `dsc`.

Specs marked ⭐ are **field-driven** - they came from real-world use and are indexed in [from-the-field.md](from-the-field.md). They carry captured API call signatures and should generally outrank speculative items.

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
- [x] ⭐ **Config-path resolution** - `$DSC_CONFIG` and `$DSC_CONFIG_HOME` env vars, explicit-selector error semantics, `dsc config` source labelling. Spec: [spec/config-path-resolution.md](config-path-resolution.md)
- [x] `dsc tag rename <discourse> <old> <new>` - in-place rename preserving every topic association; pre-flight validates existence, name collisions, and slug shape
- [x] Fix `dsc harden` test - `drop_in_uses_modern_algorithm_pins` rewritten to verify the overlay model (commit `979c3d1`)

## In progress

_(nothing currently in progress)_

## Pre-1.0 launch checklist

Polish items to land before announcing on [meta.discourse.org](https://meta.discourse.org). Most are small but cumulatively shift perception from "promising 0.x" to "stable, take it seriously."

- [ ] **Bump to 1.0.0** with a written back-compat policy. State: "the CLI surface documented in `dsc --help` is stable; flags will not be removed without a deprecation cycle." The current 0.x signal undersells the project's maturity (125 tests, 5-target prebuilt distribution, 9 months of consistent shipping).
- [ ] **Generate `CHANGELOG.md`** from `git log` (the conventional-commits style used since v0.9 makes this near-automatic). Mandated by [spec/spec.md](spec.md) but never created. Maintain going forward. Recommended approach: [`git-cliff`](https://github.com/orhun/git-cliff) (Rust-native, conventional-commits aware, additive — sits cleanly alongside the existing `s/version++` + `cargo-dist` flow). Bonus: `cargo-dist` already reads `CHANGELOG.md` to populate the GitHub Release body (see [.github/workflows/release.yml](../.github/workflows/release.yml) header), so adding the file lights up better release notes for free.
- [ ] **CLI consistency audit** against [spec/spec.md](spec.md). 30-minute pass:
  - Every `* list` command supports `--format text|json|yaml` at minimum.
  - Empty-list output matches the spec (`No <resource> found.` in text mode; empty array/object in structured modes).
  - Error messages follow the documented `discourse not found: {name}` / `{resource} not found: {identifier}` shapes.
- [ ] **Move partial-implementation metrics to a "Planned" subsection** in [docs/analytics.md](../docs/analytics.md). `lost_regulars` and `top_10_share` currently print `— (n/i)` and shouldn't sit alongside working metrics in public docs.
- [ ] **Rename [spec/dsc-tag-sync-spec.md](dsc-tag-sync-spec.md)** to `spec/tag-sync.md` for consistency with the post-`-spec.md` convention. Update any references.
- [ ] **Record an asciinema** (~30s) of the pull → edit → push → diff loop on a real Discourse. Embed in README. Visual proof beats prose.
- [ ] **"What works / what's coming" matrix in README** so readers can self-sort whether `dsc` covers their use case before installing.
- [ ] **GitHub issue templates** (`.github/ISSUE_TEMPLATE/bug_report.md`, `feature_request.md`). Inbound from Meta will not all be high-quality; templates filter the noise.
- [ ] **Decide and write down a support stance.** "Best-effort, no SLA, community-driven, see CONTRIBUTING.md" is fine. Just say it.
- [ ] **CONTRIBUTING.md** if not present. Reference [spec/implementation.md](implementation.md) and [AGENTS.md](../AGENTS.md).
- [ ] **`s/` script directory naming** - either rename to `scripts/` (conventional) or document its purpose prominently in [docs/development.md](../docs/development.md). Same for `wix/` (MSI build artefacts - obvious from contents but not from name).
- [ ] **Pre-circulate the Meta post** to one or two Discourse community regulars before posting publicly. Sanity-check framing.

## Planned

### Other planned items

- [ ] Doc accuracy pass - verify remaining docs match CLI reality
- [ ] ⭐ **Theme management gaps** - component settings, enable/disable + attach/detach, per-field editing, asset binding, `theme show`/`theme update`. Spec: [spec/theme-management.md](theme-management.md)
  - [ ] Phase 1: `dsc theme setting` (get/set/list) + `dsc theme enable|disable|attach|detach`
  - [ ] Phase 2: `dsc theme field pull/push` + `dsc theme asset set/list`
  - [ ] Phase 3: `dsc theme show` + `dsc theme update` (remote component refresh)

## Out of scope / removed

- ~~Shell completion regeneration~~ - `completions/` is gitignored and `dsc completions <shell> -d ./completions` regenerates on demand. Not a release-tracked item.
