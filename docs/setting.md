# dsc setting

Get and set site settings on a Discourse install. Requires an admin API key and username.

## dsc setting list

```
dsc setting list <discourse> [--format text|json|yaml]
```

Lists all site settings (name and value only). For a richer snapshot including defaults, descriptions, and types, see the planned `dsc setting pull` (below).

## dsc setting get

```
dsc setting get <discourse> <setting>
```

Gets the value of a site setting. Output is the raw value on stdout, suitable for piping.

## dsc setting set

```text
dsc setting set <discourse> <setting> <value>
```

Updates a site setting.

Add `--dry-run` (or `-n`) to preview the change without sending it.

## dsc setting pull

```
dsc setting pull <discourse> [path] [--changed-only] [--category <cat>]
```

Snapshot all site settings to a local YAML (or JSON, by extension) file with full metadata: `default`, `type`, `category`, `description`. The file is self-documenting and stable-sorted by category then name.

- Default path: `settings.yaml`.
- `--changed-only` (`-c`): only include settings whose value differs from default. Produces a manageable file (~50-100 entries) suitable for version control.
- `--category <cat>`: limit to a single category (e.g. `required`, `email`, `security`).

See [spec/setting-sync.md](https://github.com/pacharanero/dsc/blob/main/spec/setting-sync.md) for the schema and intended workflow.

## Planned: push and diff

The remaining phases of the bulk-management feature:

- `dsc setting push <discourse> <path>` - idempotent reconciliation. Only sends PUTs for changed values. `--dry-run` shows the plan; `--reset-unlisted` resets server settings absent from the file to their defaults.
- `dsc setting diff <a> <b>` - compare two instances or two snapshot files.
