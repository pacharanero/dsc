# dsc setting

Get and set site settings on a Discourse install. Requires an admin API key and username.

## dsc setting list

```
dsc setting list <discourse> [--format text|json|yaml]
```

Lists all site settings (name and value only). For a richer snapshot including defaults, descriptions, and types, see `dsc setting pull` below.

## dsc setting get

```
dsc setting get <discourse> <setting>
```

Gets the value of a site setting. Output is the raw value on stdout, suitable for piping.

## dsc setting set

```text
dsc setting set <discourse> <setting> <value>
dsc setting set --tags <tag1,tag2,...> <setting> <value>
```

Updates a site setting. With `--tags`, applies the change to every configured Discourse whose tag list matches any of the supplied tags (comma- or semicolon-separated). When `--tags` is given, omit `<discourse>` and pass only `<setting> <value>` as positionals.

Add `--dry-run` (or `-n`) to preview the change without sending it.

Examples:

```bash
dsc setting set myforum title "New Title"
dsc setting set --tags production,client-a login_required true
dsc -n setting set --tags staging max_invites_per_day 50   # preview
```

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

## dsc setting diff

```
dsc setting diff <source> <target> [--changed-only] [--category <cat>] [--format text|json|yaml]
```

Compare site settings between two sources. Each source can be either a Discourse name (live fetch) or a path to a snapshot file produced by `dsc setting pull`. The form is detected automatically: if the argument refers to an existing file or has a `.yaml`/`.yml`/`.json` extension, it is read as a snapshot; otherwise it is treated as a Discourse name.

Three modes:

- live vs live: `dsc setting diff staging production`
- live vs file: `dsc setting diff production prod-snapshot.yaml`
- file vs file: `dsc setting diff staging-snapshot.yaml production-snapshot.yaml`

Flags:

- `--changed-only` (`-c`): only show settings where at least one side differs from default. Recommended when one side is a `--changed-only` snapshot - otherwise the diff is dominated by entries the snapshot omitted.
- `--category <cat>`: limit to settings in one category.
- `--format` (`-f`): `text` (default), `json`, or `yaml`. JSON/YAML output is pipe-friendly for further tooling.

The text output lists each differing setting with both values quoted, or `(absent)` when one side does not have the setting at all.
