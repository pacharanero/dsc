# Roadmap

Tracks planned, in-progress, and completed work items for `dsc`.

Specs marked ⭐ are **field-driven** - they came from real-world use and are indexed in [from-the-field.md](from-the-field.md). They carry captured API call signatures and should generally outrank speculative items.

## Completed

- [x] `dsc tag pull/push` - declarative tag taxonomy management. Spec: [spec/tag-sync.md](tag-sync.md)
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
- [x] ⭐ **`dsc topic pull --full`** - full-thread Markdown snapshot with YAML frontmatter and per-post headings; batch-fetches via `/t/{id}/posts.json?post_ids[]=…`. Spec: [spec/topic-pull-full-thread.md](topic-pull-full-thread.md). Phase 2 (`--since`, `--format json`) still planned.
- [x] ⭐ **Fix `dsc user list` parse failure on negative IDs** - Discourse's built-in `system`/`discobot` accounts use IDs `-1`/`-2`, which broke deserialisation on any listing page that contained them. Widened `UserSummary.id`, `UserDetail.id`, and every user-action helper signature from `u64` to `i64`. Spec: [spec/user-list-negative-ids.md](user-list-negative-ids.md). Regression-tested.

## In progress

_(nothing currently in progress)_

## Pre-1.0 launch checklist

Polish items to land before announcing on [meta.discourse.org](https://meta.discourse.org). Most are small but cumulatively shift perception from "promising 0.x" to "stable, take it seriously."

- [ ] **Bump to 1.0.0** with a written back-compat policy. State: "the CLI surface documented in `dsc --help` is stable; flags will not be removed without a deprecation cycle." The current 0.x signal undersells the project's maturity (125 tests, 5-target prebuilt distribution, 9 months of consistent shipping).
- [x] **Generate `CHANGELOG.md`** - [git-cliff](https://github.com/orhun/git-cliff) configured via [cliff.toml](../cliff.toml); full history (back to first conventional commit) backfilled into [CHANGELOG.md](../CHANGELOG.md). `s/version++` now refreshes it automatically on each bump. `cargo-dist` picks it up for the GitHub Release body.
- [x] **CLI consistency audit** against [spec/spec.md](spec.md):
  - Format baseline: all 20 list commands accept `--format text|json|yaml` (fully compliant).
  - Empty-list text mode: 5 commands fixed to the `No <resource> found.` shape (`api-key list`, `emoji pull`, `pm list`, `search`, `tag list`). Context preserved where useful (e.g. `No PMs found in {direction}.`).
  - Error messages: `dsc tag rename` switched from `"tag 'foo' not found on 'bar'"` to the shared `not_found("tag", &old_norm)` helper, matching the `{resource} not found: {identifier}` shape used elsewhere.
- [x] **Surface analytics v1 status more prominently** in [docs/analytics.md](../docs/analytics.md). The Sections heading now carries a v1-status callout pointing at the implementation matrix and the spec's "Implementation follow-ups". Stale `.marcus/queries.md` reference fixed.
- [x] **Rename `spec/dsc-tag-sync-spec.md`** → [spec/tag-sync.md](tag-sync.md) for consistency with the post-`-spec.md` convention. References updated.
- [ ] **Record an asciinema** (~30s) of the pull → edit → push → diff loop on a real Discourse. Embed in README. Visual proof beats prose.
- [ ] **"What works / what's coming" matrix in README** so readers can self-sort whether `dsc` covers their use case before installing.
- [x] **GitHub issue templates** - [bug_report.md](../.github/ISSUE_TEMPLATE/bug_report.md), [feature_request.md](../.github/ISSUE_TEMPLATE/feature_request.md), [spec_request.md](../.github/ISSUE_TEMPLATE/spec_request.md), plus [config.yml](../.github/ISSUE_TEMPLATE/config.yml) pointing general Discourse questions at Meta.
- [x] **CONTRIBUTING.md** - lands at [CONTRIBUTING.md](../CONTRIBUTING.md), references AGENTS.md, spec/spec.md, and spec/implementation.md.
- [x] **Support stance written down** - in CONTRIBUTING.md: "best-effort, community-driven, no SLA; field-driven specs prioritised over speculative ones."
- [ ] **`s/` script directory naming** - either rename to `scripts/` (conventional) or document its purpose prominently in [docs/development.md](../docs/development.md). Same for `wix/` (MSI build artefacts - obvious from contents but not from name).
- [ ] **Pre-circulate the Meta post** to one or two Discourse community regulars before posting publicly. Sanity-check framing.
- [x] **Man page generation** via [`clap_mangen`](https://docs.rs/clap_mangen) - new `dsc man --dir <path>` subcommand emits 103 ROFF pages (root + every nested subcommand), `git`/`cargo`-style hyphen-joined filenames. Recommended workflow for distro packagers documented at [docs/manpages.md](../docs/manpages.md).
- [ ] **Evaluate `dsc open` and `dsc import`** - keep, deprecate, or document why they earn their keep before locking the CLI surface.

## Planned

- [ ] ⭐ **`category pull/push` workflow gaps + silent push** — five gaps; phases 1–4 + 6 implemented (unreleased). Only Phase 5 (MkDocs↔Discourse content conversion) remains. Spec: [spec/category-workflow.md](category-workflow.md)
  - [x] Phase 1: `category pull` writes YAML front matter (`title`, `topic_id`, `url`, `pulled_at`) + `strip_frontmatter()` in `utils.rs`
  - [x] Phase 2: `category push` routes by front-matter `topic_id`; strips front matter before sending body (also `topic push`)
  - [x] Phase 3: working `--dry-run` for `category push` with `~`/`+`/`=` output
  - [x] Phase 4: `--updates-only` flag errors instead of silently creating on mismatch
  - [ ] Phase 5: `--convert-admonitions` and `--rewrite-links` flags on push/pull (MkDocs↔Discourse portability)
  - [x] Phase 6: `--no-bump` (suppress activity-feed bump on edit) and `--skip-revision` on `topic push` / `category push`; `PostEditOptions` threaded through `update_post()`

### CLI papercuts and finishing touches

- [ ] **Universal JSON output** - the few mutating commands that still produce only single-value text (`setting get`, `theme duplicate`, `topic reply/new`) should accept `--format json|yaml`. Already easy to pipe; revisit when a user reports the papercut.
- [ ] **`palette` → `theme palette`** with a deprecation alias. Lower priority; treat as a focused patch.
- [ ] **Emoji filename preservation** - bulk uploads via `dsc emoji push <dir>` currently rename `google-drive.svg` to `google_drive` (Discourse normalises). Predictable behaviour would preserve the stem-minus-extension as the emoji name where Discourse permits.
- [ ] **`api-key create --scope <scopes>`** - scoped admin API keys (e.g. `--scope topics:write,users:read`). The existing `dsc api-key create` only mints full-admin keys.

### New command surfaces

- [ ] ⭐ **Theme management gaps** - component settings, enable/disable + attach/detach, per-field editing, asset binding, `theme show`/`theme update`. Spec: [spec/theme-management.md](theme-management.md)
  - [ ] Phase 1: `dsc theme setting` (get/set/list) + `dsc theme enable|disable|attach|detach`
  - [ ] Phase 2: `dsc theme field pull/push` + `dsc theme asset set/list`
  - [ ] Phase 3: `dsc theme show` + `dsc theme update` (remote component refresh)
- [ ] **`dsc chat`** - Discourse Chat is core now and the API is there. Subcommands: `chat channels`, `chat send <discourse> <channel> [<file>]`, `chat fetch <channel> [--since …]`. Mirrors the existing `dsc topic`/`pm` split.
- [ ] **`dsc install <name> --host <host>`** - declarative Discourse provisioning on a `dsc harden`-prepared box. Spec: [spec/install.md](install.md). Includes: templated `app.yml`, `launcher bootstrap + start`, polls `/about.json` until ready, appends the new install to `dsc.toml`. Companion to `dsc harden` (the substrate) and `dsc update` (the steady-state).
- [ ] **`dsc harden` stage 3 finishing items** - timezone/swap/journald/unattended-upgrades/fail2ban/rootless-Docker/ufw. Config keys are already wired in [src/commands/harden.rs](../src/commands/harden.rs); remaining work is the SSH-side execution and tests. See [spec/install.md](install.md) for gotchas (rootlesskit `cap_net_bind_service`, `loginctl enable-linger`, cloud firewall caveat).
- [ ] **Config schema additions for `dsc install`** - add `ssh_user: Option<String>` and `ssh_port: Option<u16>` to `DiscourseConfig`, written by `dsc install` on success. Today only `HardenConfig` carries `ssh_port`; the per-Discourse field is missing.

### Admin depth (release driven by demand)

- [ ] **`dsc log staff <discourse> [--since 7d] [--format json]`** - the staff action log.
- [ ] **`dsc report <discourse> <report-name> [--period 30d]`** - dashboard reports (signups, DAU, posts, likes). Scriptable admin dashboard. Distinct from `dsc analytics` (curated multi-metric snapshot).
- [ ] **`dsc webhook list|create|delete|ping`** - manage the plumbing automation depends on.
- [ ] **`dsc notification list|read <discourse>`** - your own notifications as the API user.

### Cross-forum specialties (the multi-install headline)

- [ ] **`dsc search all <query>`** - fan out search across every configured forum, merged results.
- [ ] **`dsc report all <name>`** - aggregate a given report across forums (e.g. total signups last 30 days across N installs).
- [ ] **`dsc setting audit <key>`** - show the current value of a given setting across every forum, diff-friendly. Distinct from `dsc setting diff` (two specific sources, all keys).
- [ ] **`dsc user find <email>`** - locate a user across every configured forum (GDPR / "which of my forums has this person" workflows).
- [ ] **`dsc backup create --all`** - reuse the parallel-ops pattern established by `dsc update all`.

### Doc accuracy

- [ ] Doc accuracy pass - verify remaining docs match CLI reality.

## Stretch / exploratory

Speculative ideas. Build only if real demand surfaces; none are required for 1.0.

- [ ] **MCP server mode** - `dsc mcp serve` exposing every command as an MCP tool, letting LLM agents drive Discourse via this CLI. Overlaps with the existing `discourse-bawmedical-mcp` - worth a think about consolidation vs coexistence.
- [ ] **TUI** - `dsc tui` for interactive browsing of forums/topics/users. Big scope.
- [ ] **Config file federation** - support multiple config files and include-directives, for teams.
- [ ] **Discourse User API** - alternative auth path for non-admins and scoped bots:
  - `dsc login <discourse> [--scopes read,write,…]` runs the full key-exchange (RSA keypair, browser to `/user-api-key/new?…`, transient-localhost callback or manual paste, decrypt, write `user_api_key` into `dsc.toml`).
  - Client emits `User-Api-Key` header when configured, preferring it over `Api-Key`/`Api-Username`.
  - Likely to require renaming the existing `dsc api-key` to `dsc admin-key` (with deprecation alias) so `dsc user-key` or similar can sit alongside.
  - **Tradeoff:** widens the *audience* (non-admins, scoped bots) but not the *capability* - most current `dsc` value (suspend, group admin, settings, backups) requires admin scope regardless.

## Out of scope / removed

- ~~Shell completion regeneration~~ - `completions/` is gitignored and `dsc completions <shell> -d ./completions` regenerates on demand. Not a release-tracked item.
- ~~`dsc user password change`~~ - dropped. Discourse's API doesn't expose an admin "set this password directly" endpoint on purpose (admins shouldn't know user passwords). `dsc user password-reset` covers the operational need.
- ~~`dsc user anonymize`~~ - dropped. Rare enough that the Admin UI is fine; not worth the destructive-confirmation UX.
