# `dsc theme` - management gaps spec

Spec for extending `dsc theme` to cover component configuration, enable/disable, per-field editing, and asset binding. Goal: drive a Discourse theme/component setup end-to-end from the CLI, without dropping into the Admin UI. Motivated by the ACCM (kitchen.culinarymedicine.org) header customisation work, where configuring header-nav components and iterating on theme SCSS from `dsc` is currently impossible.

## Context

Discourse distinguishes:

- **Themes** - top-level, user-selectable, can be the site default.
- **Components** - themes with `component: true`, attached to one or more parent themes via the parent's `child_theme_ids`. A component only takes effect while attached to an enabled parent.

A theme/component carries:

- **settings** - typed key/value pairs declared in the component's `settings.yaml`, edited per-install. This is how nav components (Custom Header Links, Dropdown Header, Header Submenus) store their menu items. Exposed in the theme JSON as `settings`; written via `PUT /admin/themes/:id/setting.json` with `name` + `value`.
- **theme_fields** - the source assets: `common/scss`, `desktop/scss`, `mobile/scss`, `extra_js`, `migrations`, translations, plus `theme_upload_var` fields that bind an uploaded image to an SCSS variable (e.g. `$logo`). Read in full by the theme JSON; written via `PUT /admin/themes/:id.json` with a `theme_fields` array.

(Exact endpoints above are the Admin UI's current behaviour and should be reconfirmed against the running Discourse version during implementation - the theme admin API is not formally versioned.)

## Current state (as of 2026-06-09)

`dsc theme` has: `list`, `install`, `remove`, `pull`, `push`, `duplicate`.

- `pull`/`push` operate on the **whole-theme JSON**. Good for backup/migration, clumsy for iterating one SCSS field.
- For locally-authored themes (e.g. ACCM's `kitchen-customisations`, id 11) `pull` returns real `theme_fields` values. For git-backed remote components (e.g. Brand Header, Header Submenus) the field values come back empty, since their source lives in the upstream repo - so `push` is not the edit path for those.
- There is **no** way to: read or write a component's **settings**; enable/disable a theme or attach/detach a component; edit a single field; or bind an uploaded image as a theme asset.

Key gaps, in priority order below.

## Phase 1 - blocking (component config + enable/disable)

### `dsc theme setting`

Read and write a single theme/component's settings (distinct from `dsc setting`, which is site settings).

```
dsc theme setting list <discourse> <theme-id> [--format text|json|yaml]
dsc theme setting get  <discourse> <theme-id> <key>
dsc theme setting set  <discourse> <theme-id> <key> <value>   [--dry-run]
```

- `list`/`get` read from the theme JSON `settings` array (`GET /admin/themes/:id.json`).
- `set` -> `PUT /admin/themes/:id/setting.json` with `name=<key>`, `value=<value>`. Honour `--dry-run`/`-n` like `dsc setting set`.
- Many nav components encode lists as a single delimited string (e.g. Header Submenus uses `|`-separated rows). `set` writes the raw string as given; documenting the per-component encoding is the user's job, not `dsc`'s.

Optional later: `dsc theme setting pull/push <theme-id> [file]` to snapshot a component's settings to YAML, mirroring `dsc setting pull/push`. Not required for the immediate work.

### `dsc theme enable` / `disable` (and component attachment)

```
dsc theme enable   <discourse> <theme-id>
dsc theme disable  <discourse> <theme-id>
dsc theme attach   <discourse> <parent-id> <component-id>     [--dry-run]
dsc theme detach   <discourse> <parent-id> <component-id>     [--dry-run]
```

- `enable`/`disable` -> `PUT /admin/themes/:id.json` toggling the theme's enabled state.
- `attach`/`detach` -> `PUT /admin/themes/:parent-id.json` adding/removing `component-id` in the parent's `child_theme_ids`. This is what actually makes a component active on a given theme, and what `dsc theme install` currently leaves to the user.
- Confirm during implementation whether "retiring" a component is best modelled as disable, or detach-from-parent. The ACCM case (Header Submenus showing unwanted demo content) is satisfied by either.

## Phase 2 - iteration ergonomics

### `dsc theme field`

Edit one `theme_field` without a whole-theme round-trip.

```
dsc theme field list <discourse> <theme-id>
dsc theme field pull <discourse> <theme-id> <target/name> [local-path]
dsc theme field push <discourse> <theme-id> <target/name> <local-path>   [--dry-run]
```

- `<target/name>` e.g. `common/scss`, `desktop/scss`, `mobile/scss`.
- `pull` writes the raw field body (e.g. the SCSS) to a file with a sensible default name; `push` PUTs just that field back via `PUT /admin/themes/:id.json` with a single-entry `theme_fields` array.
- Refuse (with a clear message) to push to a field on a git-backed remote component, where the DB value is not the source of truth.

### `dsc theme asset`

Upload an image and bind it to a theme upload variable in one step, so SCSS/settings can reference `$name`.

```
dsc theme asset set <discourse> <theme-id> <name> <file>   [--dry-run]
dsc theme asset list <discourse> <theme-id>
```

- Uploads `<file>` (reusing the existing `dsc upload` path), then writes a `theme_upload_var` `theme_field` named `<name>` bound to the resulting upload. Needed for ACCM's centred-logo image and brand imagery.
- Note: the site-wide header logos (`logo`, `logo_small`, `mobile_logo`) are **site settings**, already settable via `dsc setting set` + `dsc upload`; `theme asset` is specifically for theme-scoped `$var` assets.

## Phase 3 - nice to have

- **`dsc theme show <discourse> <theme-id>`** - richer than `list`: component flag, enabled state, default flag, parent(s), attached children, settings count, field inventory. `list` today shows only `id - name - enabled/disabled`.
- **`dsc theme update <discourse> <theme-id>`** - pull an installed *remote* component to its latest upstream commit (distinct from `dsc update`, which is the OS/Discourse rebuild). Maps to the Admin UI "check for updates" on a remote theme.

## Reference: API calls observed in the field

These are the exact Discourse admin API calls used to do this work by hand on kitchen.culinarymedicine.org while `dsc` lacked the commands. Tested against **Discourse 2026.6.0-latest** (the new glimmer header). All requests carry `Api-Key: <redacted>` and `Api-Username: <admin>` headers. They are the ground truth the proposed subcommands should wrap.

**List themes (find default, components, relationships)** - backs `theme show` / a richer `theme list`:

```
GET /admin/themes.json
```

Response: `{ "themes": [ { "id", "name", "component": bool, "default": bool, "enabled": bool, "child_themes": [{id,name}], "parent_themes": [{id,name}] }, ... ] }`. Components attach to a parent via the parent's children, not a flag on the child.

**Read one theme: settings schema + fields** - backs `theme setting list/get`, `theme field`:

```
GET /admin/themes/:id.json
```

Response `theme.settings[]` entries look like `{ "setting": "links_position", "type": "enum", "default": "right", "value": "right", "choices": [...] }`. Note: JSON-schema list settings (e.g. Dropdown Header's `header_links`) report **`"type": "string"`** here - the `json_schema` lives in the component's `settings.yml`, not the API response. The stored `value` is the JSON array serialised as a string.

**Set a theme/component setting** - backs `theme setting set`:

```
PUT /admin/themes/:id/setting.json
Content-Type: application/x-www-form-urlencoded
name=links_position&value=left
```

For a JSON-schema string setting, `value` is the JSON text, urlencoded, e.g.:

```
name=header_links&value=[{"id":1,"title":"Education","icon":"","url":"https://...","newTab":true}]
```

Returns 200 on success; 422 with a validation message if the value violates the setting's `json_schema`. No response body needed.

**Import a component from a git repo** - backs `theme install` over the API (today's `dsc theme install` is SSH-only):

```
POST /admin/themes/import.json
Content-Type: application/x-www-form-urlencoded
remote=https://github.com/paviliondev/discourse-dropdown-header&branch=main
```

Response: `{ "theme": { "id": 14, "name": "Dropdown Header", "component": true, ... } }`. `branch` is required-ish; try the repo default (`main`, then `master`).

**Attach a component to a parent theme** - backs `theme attach/detach`:

```
PUT /admin/themes/:parent-id.json
Content-Type: application/json
{ "theme": { "child_theme_ids": [8,3,1,5,13,10,4,11,6,14] } }
```

The list **replaces** the parent's children, so read the current `child_themes` from `GET /admin/themes/:parent-id.json` first and send the full set plus the new id. Disabled components stay in the list (disabled != detached).

**Enable / disable a theme or component** - backs `theme enable/disable`:

```
PUT /admin/themes/:id.json
Content-Type: application/json
{ "theme": { "enabled": false } }
```

(Confirmed via the Admin UI toggle; the `enabled` boolean round-trips in `GET /admin/themes/:id.json`.)

## Out of scope

- Authoring component source (SCSS/JS) beyond editing existing fields - that belongs in the component's own repo, not `dsc`.
- A full theme settings-diff across instances (the `dsc setting diff` analogue for themes). Could follow if `theme setting pull/push` lands.

## ACCM driver (why now)

The kitchen.culinarymedicine.org header rework needs to, from the CLI: disable/retire the Header Submenus component, configure a header-nav component's menu items (its settings), iterate `kitchen-customisations` (id 11) Common SCSS for the centred-logo layout, and bind a centred-logo image asset. Phases 1-2 cover all of that; until they land the same actions are done via the Admin UI or direct API calls.
