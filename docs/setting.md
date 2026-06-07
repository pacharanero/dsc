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

## dsc setting push

```
dsc setting push <discourse> <path> [--reset-unlisted] [--dry-run]
```

Apply a settings snapshot file to a Discourse. Idempotent: only PUTs values that differ from the current server state.

- Reads YAML or JSON (detected by extension).
- Settings present in the file but unknown on the server are skipped with a warning (handles version drift).
- `--reset-unlisted`: for settings present on the server but absent from the file, reset them to their default value. Off by default - the file describes only the values you care about.
- `--dry-run` (`-n`): print the plan without applying. Output uses `~` for change, `=` for unchanged, `?` for unknown, `-` for reset-to-default.

Example dry-run output:

```text
[dry-run] Setting push plan for prod: 2 changes, 11 unchanged, 0 unknown
  = allowed_iframes: (unchanged)
  ~ title: "Old Title" → "New Title"
  ~ login_required: "false" → "true"
```

## Planned: diff

The remaining phase of the bulk-management feature:

- `dsc setting diff <a> <b>` - compare two instances or two snapshot files.
