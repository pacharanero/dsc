# `dsc notification list|read` - notification inspection and marking read

> **Status: implemented.** Read/mark-read access to the API user's own
> Discourse notifications, via the standard user-facing `/notifications.json`
> endpoints. Roadmap item R18.

Driver: `dsc` already reads staff audit trails (`dsc log staff`); the natural
companion is the notification inbox behind the configured API
key/username itself, so an operator (or automation acting as a bot account)
can check and triage it without opening the web UI.

## Command surface

```
dsc notification list <discourse> [--filter read|unread] [--type <names>]
                                   [--limit <n>] [--format text|json|yaml]

dsc notification read <discourse> (--id <id> | --type <names> | --all)
```

`dsc notification list` filters are optional and AND together, mirroring
`NotificationsController#index`'s non-`recent` pagination mode:

- `--filter` - `read` or `unread`. Any other value is rejected client-side;
  Discourse itself silently ignores unrecognised values, which would
  otherwise return an unfiltered page and look like the flag worked.
- `--type` - comma-separated `Notification.types` symbolic names (e.g.
  `liked,mentioned`), matching Discourse's own `filter_by_types` param
  exactly (same param name, same comma-joined symbol list). Each name is
  validated against the built-in type table before requesting; Discourse
  raises a 400 on an unrecognised type; catching it client-side gives a more
  specific error before the request is sent. Custom/plugin notification
  types are not supported in this phase.
- `--limit` - newest-first rows to fetch, default 30, validated to `1..=60`
  (Discourse's own `NotificationsController::INDEX_LIMIT`) rather than
  silently clamped. When the response reaches the requested limit, `dsc`
  writes a stderr warning that older matching notifications may exist.

`dsc notification read` requires exactly one selector:

- `--id` - mark this single notification ID read (`PUT
  /notifications/mark-read.json?id=<id>`).
- `--type` - mark every unread notification of these comma-separated type
  names read (`PUT /notifications/mark-read.json?dismiss_types=<names>`).
  Validated the same way as `list`'s `--type`.
- `--all` - mark every unread notification read (`PUT
  /notifications/mark-read.json`, no params - Discourse's own "mark all
  read" behaviour).

Passing zero or more than one of `--id`/`--type`/`--all` is a validation
error before any request is sent. Honours `--dry-run`.

This first cut makes one `list` request only; it does not paginate through
older notifications. Fleet-wide aggregation is out of scope for this
command.

## Output

- **text** (default): one line per notification - timestamp, ID, read/unread
  state, symbolic type name, acting user (`-` when absent), title.
- **json** / **yaml**: the full notification list, one object per row: `id`,
  `notification_type` (Discourse's numeric `Notification.types` value),
  `read`, `high_priority`, `created_at`, `topic_id`, `post_number`,
  `fancy_title`, `slug`, `acting_user_name`, `is_warning`, `data` (a JSON hash
  whose shape depends on `notification_type`). Field set matches Discourse's
  `NotificationSerializer` minus `external_id` (Discourse Connect SSO only)
  and `acting_user_avatar_template` (not needed outside a UI).

## Notes

- Works with any valid API key/username pair - `/notifications.json` and
  `/notifications/mark-read.json` are ordinary user-facing routes, not
  `/admin/*`. `dsc` acts as the configured `api_username`, so these are that
  account's own notifications.
- Not yet covered: custom/plugin notification types, pagination past the
  first page, and per-user notification inspection (only the API user's own
  inbox is reachable) - add on demand.
