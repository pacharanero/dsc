# Roadmap

Planned and in-progress work for `dsc`. Shipped history lives in [CHANGELOG.md](../CHANGELOG.md); field-driven specs (⭐) are indexed in [from-the-field.md](from-the-field.md) and should generally outrank speculative items.

## Shipped (highlights)

The built surface, grouped - see CHANGELOG for the full per-release detail.

- **Declarative sync** - `setting pull/push/diff`, `tag pull/push`/`rename`, `category pull/push` (front-matter routing, `--dry-run`, `--updates-only`, `--no-bump`/`--skip-revision`), plus `post`/`backup`/`emoji`/`topic` pull/push. Specs: [setting-sync](setting-sync.md), [tag-sync](tag-sync.md), [category-workflow](category-workflow.md).
- **Theme management (complete)** - settings (incl. `pull/push`), fields (SCSS/HTML), assets (`set/unset`), enable/disable, attach/detach, palettes, `show`, remote `update`, API `install`/`delete`. Spec: [theme-management](theme-management.md).
- **Compliance / cross-forum** - ⭐ `sar` (GDPR SAR export), `setting audit` (one setting across the fleet). Spec: [subject-access-request](subject-access-request.md).
- **Content** - ⭐ `topic pull --full`, ⭐ `topic title`/`tags`, negative-ID user-list fix. Specs: [topic-pull-full-thread](topic-pull-full-thread.md), [topic-title-and-tags](topic-title-and-tags.md), [user-list-negative-ids](user-list-negative-ids.md).
- **Ops** - `update` (skip-if-current, rootless Docker, parallel), `harden` (PQ-hybrid SSH), ⭐ `backup setup-s3` Phase 1. Spec: [backup-s3-setup](backup-s3-setup.md).
- **CLI / distribution** - universal `--format`, `completions install` (+ PowerShell), `man` pages, `version --format`, SIGPIPE-safe piping, config-path resolution, cargo-dist release + git-cliff changelog, `s/version++` one-command release, push/PR CI gate. Specs: [config-path-resolution](config-path-resolution.md), [cli-design](cli-design.md).

## In progress

_(nothing currently in progress)_

## Pre-1.0 launch checklist

Polish before announcing on [meta.discourse.org](https://meta.discourse.org).

- [ ] **Bump to 1.0.0** with a written back-compat policy ("the `dsc --help` surface is stable; flags won't be removed without a deprecation cycle"). 0.x undersells the maturity (213 lib tests + e2e + CI gate, 5-target distribution, 9 months shipping).
- [ ] **Record an asciinema** (~30s) of the pull → edit → push → diff loop; embed in README.
- [ ] **`s/` and `wix/` naming** - keep `s/` (house style) but document it in [docs/development.md](../docs/development.md); note `wix/` holds MSI build artefacts.
- [ ] **Pre-circulate the Meta post** to a couple of Discourse regulars before posting.
- [ ] **Evaluate `dsc open` and `dsc import`** - keep, deprecate, or justify before locking the surface.

## Planned

### `dsc update` refinements ⭐

Spec: [update-concurrency](update-concurrency.md).

- [x] **Leaner `-p [N]`** - folded `-p`/`--parallel` + `-m`/`--max` into one optional-value flag (`-p` = width 3, `-p N` = N workers); `-m` dropped. Implemented (unreleased).
- [x] **Rebuild-lock pre-flight** - skips a forum that already has a `./launcher rebuild` in flight (`pgrep -f '[l]auncher rebuild'`), *before* the reboot, so a re-run never stomps a supervised rebuild. `--force` overrides. Implemented (unreleased); verified on koloki-demo.

### Content sync

- [ ] ⭐ **`category` Phase 5** - `--convert-admonitions` / `--rewrite-links` for MkDocs↔Discourse portability (the only remaining gap; phases 1-4, 6 shipped). Spec: [category-workflow](category-workflow.md).

### New command surfaces

- [ ] **`dsc chat`** - `chat channels` / `chat send <discourse> <channel> [<file>]` / `chat fetch <channel> [--since …]`. Mirrors the `topic`/`pm` split.
- [ ] ⭐ **`backup setup-s3` Phase 2/3** - `--reuse-user` (key rotation), `--use-iam-profile`, `--all`/`--tags`; then a native AWS SDK backend, `--retention` lifecycle, `backup status`. Spec: [backup-s3-setup](backup-s3-setup.md).
- [ ] **`dsc install <name> --host <host>`** - declarative provisioning on a `dsc harden`-prepared box (templated `app.yml`, launcher bootstrap, poll `/about.json`, append to `dsc.toml`). Spec: [install](install.md). Includes the remaining `harden` stage-3 items (timezone/swap/journald/unattended-upgrades/fail2ban/rootless-Docker/ufw - config keys wired, SSH execution + tests remain) and the `ssh_user`/`ssh_port` per-Discourse config fields `install` writes on success.

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
