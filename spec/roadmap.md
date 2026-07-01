# Roadmap

Planned and in-progress work for `dsc`. Shipped history lives in [CHANGELOG.md](../CHANGELOG.md); field-driven specs (⭐) are indexed in [from-the-field.md](from-the-field.md) and should generally outrank speculative items.

## Shipped (highlights)

The built surface, grouped - see CHANGELOG for the full per-release detail.

- **Declarative sync** - `setting pull/push/diff`, `tag pull/push`/`rename`, `category pull/push` (front-matter routing, `--dry-run`, `--updates-only`, `--no-bump`/`--skip-revision`), plus `post`/`backup`/`emoji`/`topic` pull/push. Specs: [setting-sync](commands/setting-sync.md), [tag-sync](commands/tag-sync.md), [category-workflow](commands/category-workflow.md).
- **Theme management (complete)** - settings (incl. `pull/push`), fields (SCSS/HTML), assets (`set/unset`), enable/disable, attach/detach, palettes, `show`, remote `update`, API `install`/`delete`. Spec: [theme-management](commands/theme-management.md).
- **Compliance / cross-forum** - ⭐ `sar` (GDPR SAR export), `setting audit` (one setting across the fleet). Spec: [subject-access-request](commands/subject-access-request.md).
- **Content** - ⭐ `topic pull --full`, ⭐ `topic title`/`tags`, negative-ID user-list fix. Specs: [topic-pull-full-thread](commands/topic-pull-full-thread.md), [topic-title-and-tags](commands/topic-title-and-tags.md), [user-list-negative-ids](commands/user-list-negative-ids.md).
- **Ops** - `update` (skip-if-current, rootless Docker, parallel), `harden` (PQ-hybrid SSH), ⭐ `backup setup-s3` Phase 1. Spec: [backup-s3-setup](commands/backup-s3-setup.md).
- **CLI / distribution** - universal `--format`, `completions install` (+ PowerShell), `man` pages, `version --format`, SIGPIPE-safe piping, config-path resolution, cargo-dist release + git-cliff changelog, `s/version++` one-command release, push/PR CI gate. Specs: [config-path-resolution](commands/config-path-resolution.md), [cli-design](cli-design.md).

## In progress

_(nothing currently in progress)_

## Known bugs

- [ ] **`tag pull` writes tag-group permissions keyed by group *id*, not name.** A pulled `tags.yaml` shows e.g. `permissions: {'0': full}` instead of `permissions: {everyone: full}`. What's happening: Discourse's `tag_groups.json` returns each group's `permissions` as a map keyed by the numeric **group id** (`{"0": 1}` - id `0` is the built-in `everyone` group, value `1` = full), but `parse_tag_group_permissions` (`src/commands/tag.rs`) assumes the map is keyed by group **name** and copies the key through verbatim. So the id leaks into the file as if it were a group name. Pushing that file back then sends `permissions[0]=…`, and `0` is not a valid group name, so the permission does **not** round-trip. Secondary: only levels `1`/`3` are labelled (`full`/`readonly`); level `2` = `create_post` is left as the bare string `"2"`. **Impact is low today** - tag groups are usually created fresh (Discourse applies the default `everyone` permission implicitly) and dsc doesn't yet set restrictive tag-group permissions, so nobody has hit a broken restriction; it's a cosmetic-plus-latent-round-trip bug, not a data-loss one. **Fix:** resolve group id ↔ name on both sides (pull: map id→name using the group list; push: map name→id), and label level `2` as `create_post`; or, until then, drop the `permissions` field from `tag pull` output so it can't emit something that won't round-trip. Discovered 2026-07-01 on koloki-demo while verifying the tag delete/create fixes. Spec: [tag-sync](commands/tag-sync.md).

## Pre-1.0 launch checklist

Polish before announcing on [meta.discourse.org](https://meta.discourse.org).

- [ ] **Bump to 1.0.0** with a written back-compat policy ("the `dsc --help` surface is stable; flags won't be removed without a deprecation cycle"). 0.x undersells the maturity (213 lib tests + e2e + CI gate, 5-target distribution, 9 months shipping).
- [ ] **Record an asciinema** (~30s) of the pull → edit → push → diff loop; embed in README.
- [ ] **`s/` and `wix/` naming** - keep `s/` (house style) but document it in [docs/development.md](../docs/development.md); note `wix/` holds MSI build artefacts.
- [ ] **Pre-circulate the Meta post** to a couple of Discourse regulars before posting.
- [ ] **Evaluate `dsc open` and `dsc import`** - keep, deprecate, or justify before locking the surface.

## Planned

### `dsc update` refinements ⭐

Spec: [update-concurrency](commands/update-concurrency.md).

- [x] **Leaner `-p [N]`** - folded `-p`/`--parallel` + `-m`/`--max` into one optional-value flag (`-p` = width 3, `-p N` = N workers); `-m` dropped. Implemented (unreleased).
- [x] **Rebuild-lock pre-flight** - skips a forum that already has a `./launcher rebuild` in flight (`pgrep -f '[l]auncher rebuild'`), *before* the reboot, so a re-run never stomps a supervised rebuild. `--force` overrides. Implemented (unreleased); verified on koloki-demo.
- [x] ⭐ **Update log + skip-recent** - append-only TSV log of what `dsc update` did to each forum and when (`$XDG_STATE_HOME/dsc/update.log`); `dsc update log [--latest] [--since] [--format]` renders it (incl. the "Servers" checklist view). `--skip-recent [dur]` skips forums updated within a window (default 24h) so fleet re-runs don't repeat the day's work / re-reboot; interactively you're prompted up front instead. Implemented (unreleased); verified on koloki-demo. Stretch: post a summary to a home Discourse. Spec: [update-log](commands/update-log.md)

### Content sync

- [ ] ⭐ **`category` Phase 5** - `--convert-admonitions` / `--rewrite-links` for MkDocs↔Discourse portability (the only remaining gap; phases 1-4, 6 shipped). Spec: [category-workflow](commands/category-workflow.md).
- [x] ⭐ **`category` definition sync** - `category def pull/push` (declarative file round-trip of category *definitions*: description, permissions, position, topic template, tag rules) + `category show/get/set` (imperative single-field). Fills the gap beside the topic-content `category pull/push`. **Phase 1 implemented (unreleased)**, verified on koloki-demo; Phases 2-3 (rename, list `--append`/`--remove`, prune, `def diff`) planned. Spec: [category-definition-sync](commands/category-definition-sync.md).

### New command surfaces

- [ ] **`dsc chat`** - `chat channels` / `chat send <discourse> <channel> [<file>]` / `chat fetch <channel> [--since …]`. Mirrors the `topic`/`pm` split.
- [ ] ⭐ **`backup setup-s3` Phase 2/3** - `--reuse-user` (key rotation), `--use-iam-profile`, `--all`/`--tags`; then a native AWS SDK backend, `--retention` lifecycle, `backup status`. Spec: [backup-s3-setup](commands/backup-s3-setup.md).
- [ ] **`dsc install <name> --host <host>`** - declarative provisioning on a `dsc harden`-prepared box (templated `app.yml`, launcher bootstrap, poll `/about.json`, append to `dsc.toml`). Spec: [install](commands/install.md). Includes the remaining `harden` stage-3 items (timezone/swap/journald/unattended-upgrades/fail2ban/rootless-Docker/ufw - config keys wired, SSH execution + tests remain) and the `ssh_user`/`ssh_port` per-Discourse config fields `install` writes on success.

### Admin depth (demand-driven)

- [ ] **`dsc log staff`** (staff action log), **`dsc report <name> [--period]`** (dashboard reports - signups/DAU/posts/likes, distinct from `analytics`), **`dsc webhook list|create|delete|ping`**, **`dsc notification list|read`**.

### Cross-forum (the multi-install headline)

- [ ] **`dsc search all <query>`** (merged fan-out search), **`dsc report all <name>`** (aggregate a report across forums), **`dsc user find <email>`** (GDPR "which forum has this person"), **`dsc backup create --all`** (reuse the `update all` parallel pattern).

### Doc accuracy

- [ ] Pass to verify remaining docs match CLI reality.

## Stretch / exploratory

Speculative; build only on real demand. None are required for 1.0.

- [ ] **MCP server mode** - `dsc mcp serve` exposing commands as MCP tools. Overlaps `discourse-bawmedical-mcp` - consolidate vs coexist.
- [ ] **TUI** - `dsc tui` for interactive browsing. Big scope.
- [ ] **Config federation** - multiple config files + include-directives, for teams.
- [ ] **Discourse User API** - `dsc login` key-exchange for non-admin / scoped-bot auth; likely renames `api-key` → `admin-key`. Widens the *audience*, not the *capability* (most value needs admin scope regardless).

## Out of scope / removed

- ~~Shell completion *regeneration* as a tracked item~~ - superseded by the shipped `completions install`.
- ~~`dsc user password change`~~ - Discourse has no admin "set this password" endpoint by design; `user password-reset` covers the need.
- ~~`dsc user anonymize`~~ - rare enough for the Admin UI; not worth the destructive-confirmation UX.
- ~~`api-key create --scope`~~ - **parked 2026-06-29**. Scoped keys are low-value for `dsc` (nearly everything needs admin scope anyway) and blocked on an unconfirmed scoped-key `POST /admin/api/keys.json` body. Full-admin `api-key create` stays. Revisit on a concrete least-privilege consumer.
