# dsc specs

Design specs for `dsc`, in two tiers.

## Overarching (this directory)

Cross-cutting documents that sit above any single command:

- [spec.md](spec.md) - internal spec: the `dsc.toml` config schema and the release/distribution rules.
- [cli-design.md](cli-design.md) - the normative CLI design philosophy: output/formats, the `pull → edit → push → diff` sync loop, `--dry-run`, destructive-action guards, error/empty-list/flag conventions. Anything about *how commands behave* lives here.
- [implementation.md](implementation.md) - the implementation plan and the working agreement for agents (commit discipline, keeping specs current, roadmap flow).
- [roadmap.md](roadmap.md) - planned and in-progress work. Shipped history is in [CHANGELOG.md](../CHANGELOG.md).
- [from-the-field.md](from-the-field.md) - index of the field-driven (⭐) specs: the ones that came from a real task against a live Discourse, with the API calls captured in the field. These outrank speculative items.

## Per-command ([commands/](commands/))

One spec per discrete feature or gap, named after the command surface it belongs to and mirroring `src/commands/` and [docs/](../docs/). A single command can own more than one spec when the work arrived in distinct pieces - for example `dsc category` has both [commands/category-workflow.md](commands/category-workflow.md) (the pull/edit/push loop) and [commands/category-definition-sync.md](commands/category-definition-sync.md) (syncing category *definitions*). Specs stay discrete rather than being merged, so each keeps its own driver, field-API reference, and phase checklist.

## Conventions

- A spec that originated from real-world use gets a ⭐ and a row in [from-the-field.md](from-the-field.md), including a "Reference: API calls observed in the field" section (template in [../AGENTS.md](../AGENTS.md)). Record the Discourse version tested against - the admin API is not formally versioned.
- User-facing per-command usage lives in [docs/](../docs/), not here. Specs are design intent; docs are the reference.
