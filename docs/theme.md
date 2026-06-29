# dsc theme

List, install, remove, pull, push, and duplicate themes; read/write a theme's settings; enable/disable and attach/detach components.

## dsc theme list

```
dsc theme list <discourse> [--format text|json|yaml]
```

Lists installed themes on the specified Discourse.

## dsc theme install

```
dsc theme install <discourse> <url>
```

Installs a theme using the SSH command template in `DSC_SSH_THEME_INSTALL_CMD`. The template supports `{url}` and `{name}` placeholders.

## dsc theme remove

```
dsc theme remove <discourse> <name>
```

Removes a theme using the SSH command template in `DSC_SSH_THEME_REMOVE_CMD`. The template supports `{name}` and `{url}` placeholders.

## dsc theme pull

```
dsc theme pull <discourse> <theme-id> [<local-path>]
```

Pulls the specified theme into a local JSON file. `<theme-id>` can be found using `dsc theme list`.

If `<local-path>` is omitted, the file is written to the current directory named from the theme name (slugified). The path to the written file is printed to stdout.

## dsc theme push

```
dsc theme push <discourse> <json-path> [<theme-id>]
```

Pushes a local JSON theme file to a Discourse instance.

- If `<theme-id>` is supplied, updates the existing theme and prints the ID.
- If the JSON file contains an `id` field and no `<theme-id>` argument is given, updates that theme.
- Otherwise creates a new theme and prints the new ID.

## dsc theme duplicate

```
dsc theme duplicate <discourse> <theme-id> [--format text|json|yaml]
```

Duplicates the specified theme and prints the new theme ID. The copy is named `Copy of <original name>` and is not set as the default theme. `--format json` emits `{"id": ...}`.

## dsc theme show

```
dsc theme show <discourse> <theme-id> [--format text|json|yaml]
```

Shows a richer view of one theme/component than `list`: whether it's a theme or component, its enabled/default/user-selectable flags, colour scheme, parent themes, attached child components, settings count, and the editable field inventory (e.g. `common/scss`). Read-only.

```bash
dsc theme show accm 11
dsc theme show accm 11 --format json
```

## dsc theme setting

Read and write a single theme or component's **settings** - the typed key/value pairs a component declares (e.g. a nav component's menu items). Distinct from `dsc setting`, which manages site-wide settings.

```
dsc theme setting list <discourse> <theme-id> [--format text|json|yaml]
dsc theme setting get  <discourse> <theme-id> <key> [--format text|json|yaml]
dsc theme setting set  <discourse> <theme-id> <key> <value>   [--dry-run]
```

- `list` / `get` read from the theme JSON (`GET /admin/themes/:id.json`). `list` prints `key = value` per line in text mode; `json`/`yaml` include each setting's `type` and `default`.
- `set` writes via `PUT /admin/themes/:id/setting.json`. The `<value>` is sent **verbatim** - for a JSON-schema list setting (e.g. a header-links component), pass the JSON array text directly, quoted for your shell. Honours global `-n` / `--dry-run`.

```bash
dsc theme setting get accm 11 links_position
dsc theme setting set accm 14 links_position left
dsc theme setting set accm 14 header_links '[{"id":1,"title":"Education","url":"https://..."}]'
```

### dsc theme setting pull / push

For a whole component - and especially for the big JSON-list settings like a nav component's `header_links` / `dropdown_links` - snapshot the settings to a file, edit, and push back. The same pull → edit → push loop `dsc setting pull/push` gives site settings.

```
dsc theme setting pull <discourse> <theme-id> [<local-path>]
dsc theme setting push <discourse> <theme-id> <local-path>   [--dry-run]
```

- `pull` writes every setting to a file (YAML by default; a `.json` path writes JSON). **JSON-list settings are expanded to real arrays** - `header_links` becomes an editable list of entries, not one escaped string on a line. If `<local-path>` is omitted it writes `<theme-name>-settings.yml` in the current directory.
- `push` re-serialises each list back to the JSON-string form Discourse expects and **PUTs only the settings that changed** (compared semantically, so reformatting and key-order don't count as edits - an untouched pull → push is a clean no-op). Settings in the file that no longer exist on the component are skipped with a warning. `--dry-run` prints the plan, summarising long list values by length so the terminal isn't flooded.

This replaces the read-whole-array → change-one-field → PUT-it-back scripting that editing a header menu by hand otherwise needs.

```bash
# Snapshot the Dropdown Header component (id 17), edit, preview, apply
dsc theme setting pull accm 17 header.yml
$EDITOR header.yml                                  # rename "Conference 2026" -> "2027" in the list
dsc theme setting push accm 17 header.yml --dry-run # header_links: changed (864 -> 868 chars)
dsc theme setting push accm 17 header.yml
```

## dsc theme enable / disable

```
dsc theme enable  <discourse> <theme-id>
dsc theme disable <discourse> <theme-id>
```

Toggles a theme or component's enabled state (`PUT /admin/themes/:id.json`). A disabled component stays attached to its parents (disabled is not the same as detached). Honours `-n` / `--dry-run`.

## dsc theme attach / detach

```
dsc theme attach <discourse> <parent-id> <component-id>
dsc theme detach <discourse> <parent-id> <component-id>
```

Attaches or detaches a component to/from a parent theme - this is what makes a component active on a given theme (and what `dsc theme install` leaves to you). `dsc` reads the parent's current children, adds/removes the component, and writes the full replacement set, so other attached components are preserved. Detaching an id that isn't attached (or attaching one that already is) is a reported no-op. Honours `-n` / `--dry-run`.

```bash
# Make component 14 active on the default theme (id 2)
dsc theme attach accm 2 14
# Retire it again
dsc theme detach accm 2 14
```
