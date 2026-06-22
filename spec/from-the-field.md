# Specs from the field

Index of specs that originated from **real-world use** of `dsc` against a live Discourse, as opposed to features designed in the abstract. Field-driven specs are higher-confidence: the use case is real, the API surface was discovered by actually hitting the endpoints, and the priority ordering reflects what genuinely blocked someone. They should generally outrank speculative items in [roadmap.md](roadmap.md).

Each entry names the driver (the real task), the date it surfaced, and whether the spec includes a "Reference: API calls observed in the field" section (the call signatures captured from the workaround - the most valuable part for implementation).

| Spec | Driver | Surfaced | Field API calls captured | Status |
|---|---|---|---|---|
| [category-workflow.md](category-workflow.md) | playbook.rcpch.tech migration: offline Git-tracked sync of `forum.rcpch.tech/c/playbook` (category 34, 27 topics). Three gaps blocked the pull→edit→push workflow: no topic IDs in pulled files, `--dry-run` silently ignored on push, and slug mismatch causing silent topic creation. | 2026-06-22 | Yes — category topic list JSON (`/c/playbook/34.json`) and `PUT /posts/{id}.json` body format confirmed against Discourse stable | Planned |
| [config-path-resolution.md](config-path-resolution.md) | `dsc.toml` lived outside the standard search paths (in a tool repo), so `dsc` could not reach the ACCM forum from any other directory; needed an env-var override mirroring `sct`. | 2026-06-09 | n/a (local config logic, no Discourse API) | Implemented v0.10.9 |
| [theme-management.md](theme-management.md) | ACCM forum (kitchen.culinarymedicine.org) header rework: had to install + attach a component, configure its JSON-schema settings, and toggle components - none of which `dsc theme` could do. Worked around via raw admin API. | 2026-06-09 | Yes - import, attach, enable/disable, read/set theme settings, against Discourse 2026.6.0-latest | Planned |
| [topic-pull-full-thread.md](topic-pull-full-thread.md) | forum.rcpch.tech topic 364: an LLM agent needed to read every post in a 27-post thread to draft a response. `dsc topic pull` only wrote the OP; worked around via two manual `curl` calls with pagination. | 2026-06-10 | Yes - paginated `/t/{id}.json?include_raw=1`, recommended batch-fetch via `/t/{id}/posts.json?post_ids[]=…`, against Discourse 3.x | Phase 1 implemented v0.10.11 |
| [user-list-negative-ids.md](user-list-negative-ids.md) | Extracting the full member email list from restorativejustculture.org for an NHS Trust analysis: pages 4 and 5 of `dsc user ls -f json` failed to parse because Discourse's built-in `system` and `discobot` accounts use negative IDs (-1, -2), and the model typed `id` as `u64`. Worked around by calling the admin endpoint directly with `curl`. | 2026-06-17 | Yes - `/admin/users/list/active.json?show_emails=true&page=N`, against Discourse stable | Fixed in v0.10.14 |

## Adding an entry

When you write a spec because `dsc` could not do something you needed on a real install, add a row here and include a "Reference: API calls observed in the field" section in the spec itself (see the template in [../AGENTS.md](../AGENTS.md)). Record the Discourse version you tested against - the admin API is not formally versioned, so the version is part of the ground truth.
