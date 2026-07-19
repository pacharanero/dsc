# Roadmap

Planned and in-progress work for `dsc`. Shipped history lives in [CHANGELOG.md](../CHANGELOG.md); active field-driven specs (⭐) are indexed in [from-the-field.md](from-the-field.md) and should generally outrank speculative items.

Legend: [x] done, [~] in progress or partially done, [ ] not started. Stable roadmap codes (`R1`, `R2`, …) are never renumbered or reused.

## Shipped (highlights)

The built surface, grouped - see CHANGELOG for the full per-release detail.

- **Declarative sync** - `setting pull/push/diff`, `tag pull/push`/`rename`, `category pull/push` (front-matter routing, `--dry-run`, `--updates-only`, `--no-bump`/`--skip-revision`), plus `post`/`backup`/`emoji`/`topic` pull/push. Specs: [setting-sync](commands/setting-sync.md), [tag-sync](commands/tag-sync.md), [category-workflow](commands/category-workflow.md).
- **Theme management (complete)** - settings (incl. `pull/push`), fields (SCSS/HTML), assets (`set/unset`), enable/disable, attach/detach, palettes, `show`, remote `update`, API `install`/`delete`. Spec: [theme-management](commands/theme-management.md).
- **Compliance / cross-forum** - ⭐ `sar` (GDPR SAR export), `setting audit` (one setting across the fleet). Spec: [subject-access-request](commands/subject-access-request.md).
- **Content** - ⭐ `topic pull --full`, ⭐ `topic title`/`tags`, ⭐ `topic delete`/`restore`/`list --deleted`, negative-ID user-list fix. Specs: [topic-pull-full-thread](commands/topic-pull-full-thread.md), [topic-title-and-tags](commands/topic-title-and-tags.md), [topic-delete](commands/topic-delete.md), [user-list-negative-ids](commands/user-list-negative-ids.md).
- **Ops** - `update` (skip-if-current, rootless Docker, parallel), `harden` (PQ-hybrid SSH), ⭐ `backup setup-s3` Phase 1. Spec: [backup-s3-setup](commands/backup-s3-setup.md).
- **CLI / distribution** - universal `--format`, `completions install` (+ PowerShell), `man` pages, `version --format`, SIGPIPE-safe piping, config-path resolution, cargo-dist release + git-cliff changelog, `s/version++` one-command release, push/PR CI gate. Specs: [config-path-resolution](commands/config-path-resolution.md), [cli-design](cli-design.md).

### Completed roadmap items

- [x] **R1 - Tag-group permission name/ID round-trip** - taxonomy files use group names and semantic levels (`full`, `create_post`, `readonly`); `dsc` translates to and from the numeric API representation and detects permission-only updates. Spec: [tag-sync](commands/tag-sync.md).
- [x] **R7 - Leaner `-p [N]`** - folded `-p`/`--parallel` + `-m`/`--max` into one optional-value flag (`-p` = width 3, `-p N` = N workers); `-m` dropped.
- [x] **R8 - Rebuild-lock pre-flight** - skips a forum with a `./launcher rebuild` in flight before reboot; `--force` overrides.
- [x] ⭐ **R9 - Update log + skip-recent** - append-only update log plus `dsc update log` and `--skip-recent [dur]` for safe fleet re-runs. Specs: [update-concurrency](commands/update-concurrency.md), [update-log](commands/update-log.md).
- [x] **R15 - `dsc log staff`** - staff action log access: filter by `--action`, `--acting-user`, `--target-user`, `--subject`, `--since`; `--format text|json|yaml`. Spec: [staff-action-log](commands/staff-action-log.md).
- [x] **R18 - `dsc notification list|read`** - notification inspection and marking read: `list --filter read|unread --type <names> --limit`; `read --id|--type|--all`. Spec: [notification](commands/notification.md).

## In progress

- [~] ⭐ **R10 - `category` Phase 5** - `--convert-admonitions=quote-callouts|plain-blockquote` carries MkDocs/Zensical callouts to and from category-topic Markdown. The Quote Callouts target is explicit because it requires the Arkshine theme component; the plain-blockquote target is portable and email-safe. Internal `--rewrite-links` remains. Spec: [category-workflow](commands/category-workflow.md).

## Pre-1.0 launch checklist

Polish before announcing on [meta.discourse.org](https://meta.discourse.org).

- [ ] **R2 - Bump to 1.0.0** with a written back-compat policy ("the `dsc --help` surface is stable; flags won't be removed without a deprecation cycle"). 0.x undersells the maturity (213 lib tests + e2e + CI gate, 5-target distribution, 9 months shipping).
- [ ] **R3 - Record an asciinema** (~30s) of the pull → edit → push → diff loop; embed in README.
- [x] **R4 - `s/` and `wix/` naming** - keep `s/` (house style) but document it in [docs/development.md](../docs/development.md); note `wix/` holds MSI build artefacts.
- [ ] **R5 - Pre-circulate the Meta post** to a couple of Discourse regulars before posting.
- [ ] **R6 - Evaluate `dsc open` and `dsc import`** - keep, deprecate, or justify before locking the surface.

## Planned

### Docker app configuration ⭐

- [ ] ⭐ **R28 - `dsc app env` inspect, audit, and safe set** - read environment-variable names and non-secret values from Docker `app.yml`, audit one key across matching forums, then add guarded scalar edits with backup, dry-run, and optional rebuild. Driver: inspecting or raising `DISCOURSE_MAX_ADMIN_API_REQS_PER_MINUTE` across the owned fleet. Spec: [app-environment](commands/app-environment.md).

### Content sync


- [~] ⭐ **R11 - `category` definition sync Phase 2/3** - Phase 1 shipped the blocking round-trip (`category def pull/push`, `category show/get/set`) for category definitions: description, permissions, position, topic template, and tag rules. Remaining work: rename, list `--append`/`--remove`, prune, and `def diff`. Spec: [category-definition-sync](commands/category-definition-sync.md).

### New command surfaces

- [ ] **R12 - `dsc chat`** - `chat channels` / `chat send <discourse> <channel> [<file>]` / `chat fetch <channel> [--since …]`. Mirrors the `topic`/`pm` split.
- [ ] ⭐ **R13 - `backup setup-s3` Phase 2/3** - `--reuse-user` (key rotation), `--use-iam-profile`, `--all`/`--tags`; then a native AWS SDK backend, `--retention` lifecycle, `backup status`. Spec: [backup-s3-setup](commands/backup-s3-setup.md).
- [ ] **R14 - `dsc install <name> --host <host>`** - declarative provisioning on a `dsc harden`-prepared box (templated `app.yml`, launcher bootstrap, poll `/about.json`, append to `dsc.toml`). Spec: [install](commands/install.md). Includes the remaining `harden` stage-3 items (timezone/swap/journald/unattended-upgrades/fail2ban/rootless-Docker/ufw - config keys wired, SSH execution + tests remain) and the `ssh_user`/`ssh_port` per-Discourse config fields `install` writes on success.

### Admin depth (demand-driven)

- [ ] **R16 - `dsc report <name> [--period]`** - dashboard reports such as signups, DAU, posts, and likes; distinct from `analytics`.
- [ ] **R17 - `dsc webhook list|create|delete|ping`** - basic webhook administration.

### Cross-forum (the multi-install headline)

- [ ] **R19 - `dsc search all <query>`** - merged fan-out search.
- [ ] **R20 - `dsc report all <name>`** - aggregate a report across forums.
- [ ] **R21 - `dsc user find <email>`** - GDPR "which forum has this person" lookup.
- [ ] **R22 - `dsc backup create --all`** - reuse the `update all` parallel pattern for fleet backups.

### Doc accuracy

- [ ] **R23 - Docs/CLI reality pass** - verify remaining docs match the current CLI surface.

## Stretch / exploratory

Speculative; build only on real demand. None are required for 1.0.

- [ ] **R24 - MCP server mode** - `dsc mcp serve` exposing commands as MCP tools. Overlaps `discourse-bawmedical-mcp` - consolidate vs coexist.
- [ ] **R25 - TUI** - `dsc tui` for interactive browsing. Big scope.
- [ ] **R26 - Config federation** - multiple config files + include-directives, for teams.
- [ ] **R27 - Discourse User API** - `dsc login` key-exchange for non-admin / scoped-bot auth; likely renames `api-key` → `admin-key`. Widens the *audience*, not the *capability* (most value needs admin scope regardless).

## Out of scope / removed

- ~~Shell completion *regeneration* as a tracked item~~ - superseded by the shipped `completions install`.
- ~~`dsc user password change`~~ - Discourse has no admin "set this password" endpoint by design; `user password-reset` covers the need.
- ~~`dsc user anonymize`~~ - rare enough for the Admin UI; not worth the destructive-confirmation UX.
- ~~`api-key create --scope`~~ - **parked 2026-06-29**. Scoped keys are low-value for `dsc` (nearly everything needs admin scope anyway) and blocked on an unconfirmed scoped-key `POST /admin/api/keys.json` body. Full-admin `api-key create` stays. Revisit on a concrete least-privilege consumer.
