# Specs from the field

Active index of **field-driven** specs: requests that came from real use of `dsc` against a live Discourse and still have unimplemented work. These should generally outrank speculative roadmap items because the use case is proven and, where relevant, the API surface was captured from the workaround.

Completed field-driven work is removed from this file once the behaviour is documented in the relevant [docs/](../docs/) page and/or retained in the command spec. Shipped history lives in [CHANGELOG.md](../CHANGELOG.md) and the high-level status is in [roadmap.md](roadmap.md).

| Spec | Remaining field request | Surfaced | Field evidence | Status / next step |
|---|---|---|---|---|
| [category-workflow.md](commands/category-workflow.md) | MkDocs ↔ Discourse content portability for category topic sync: explicit Quote Callouts / plain-blockquote admonition conversion, then URL/link rewriting. | 2026-06-22 | Yes for the surrounding category pull/push workflow; the remaining conversion work is local content transformation, not a new API endpoint. Quote Callouts syntax verified from its Meta topic and maintained theme-component source. | Admonition conversion implemented; `--rewrite-links` remains. |
| [category-definition-sync.md](commands/category-definition-sync.md) | Iteration ergonomics beyond Phase 1: safe category rename, list-field append/remove, richer definition fields, guarded prune, `def diff`, and fuller cross-forum copy. | 2026-07-01 | Yes — `/categories.json?show_permissions=true` full category objects including `group_permissions`, `topic_template`, `position`, `allowed_tags`; tested against Discourse 2026.7.0-latest. | Phase 1 implemented; Phases 2–3 planned. |
| [backup-s3-setup.md](commands/backup-s3-setup.md) | Backup setup re-runs and fleet ergonomics: key rotation/reuse, IAM-profile mode, `--all`/`--tags`, optional native AWS SDK, lifecycle retention, and `backup status`. | 2026-06-24 | Yes — single-bucket IAM policy JSON, `aws s3api`/`aws iam` provisioning commands, and Discourse S3 backup settings from the production runbook. | Phase 1 implemented; Phases 2–3 planned. |
| [app-environment.md](commands/app-environment.md) | Inspect Docker `app.yml` environment variables and audit a key across the owned fleet, then safely change one scalar `env:` entry with backup and an optional rebuild. | 2026-07-14 | Yes — standard SSH workflow to inspect `/var/discourse/containers/app.yml` and `./launcher rebuild app`; 53-second 429 retries during `dsc tag pull dhi-discourse` identified `DISCOURSE_MAX_ADMIN_API_REQS_PER_MINUTE` as the immediate driver. | Phase 1 planned. |

## Adding an entry

When `dsc` cannot do something needed on a real install, add a row here and include a `Reference: API calls observed in the field` section in the spec itself when an API was involved. Record the Discourse version tested against; the admin API is not formally versioned, so the version is part of the ground truth.

When the blocking field need is completed, move any still-useful operational detail into the command spec and user-facing behaviour into `docs/`, then remove the row from this active index.
