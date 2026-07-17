# `dsc log staff` - staff action log access

> **Status: implemented.** Read-only access to Discourse's staff action log
> (`UserHistory`) - the audit trail behind `/admin/logs/staff-action-logs` in
> the web UI. Roadmap item R15.

Driver: "who changed this setting, and when" is a recurring question across a
multi-forum fleet, and today the only answer is clicking through the admin UI
on each forum individually. `dsc log staff` makes that a one-liner, and lets
the answer be piped, diffed, or scripted like the rest of `dsc`.

## Command surface

```
dsc log staff <discourse> [--action <name>] [--acting-user <username>]
                           [--target-user <username>] [--subject <text>]
                           [--since <dur>] [--limit <n>] [--format text|json|yaml]
```

All filters are optional and AND together, mirroring Discourse's own
`UserHistory.staff_filters`:

- `--action` - supported built-in action name, e.g. `change_site_setting`,
  `suspend_user`, `delete_post`, `grant_admin`. `dsc` validates this against
  the core staff-action names it knows before requesting logs. This is a safety
  guard: Discourse silently drops the action predicate for an unrecognised
  `action_name`, which would otherwise return unrelated newest records.
  Custom/plugin action types are deliberately not supported in this phase.
- `--acting-user` - the staff member who performed the action.
- `--target-user` - the user the action was performed on (absent for
  non-user-targeted actions, e.g. a setting change).
- `--subject` - exact match on the subject field (often a setting name). The
  Discourse endpoint uses `subject = ?`; a substring-search option needs a
  paginated client-side design and is not claimed here.
- `--since` - relative duration (`7d`, `24h`) or an ISO-8601 date/timestamp,
  parsed with the same `parse_since_cutoff` used by `analytics` and
  `update log`, then sent as a precise UTC RFC 3339 `start_date` timestamp.
- `--limit` - newest-first rows to fetch, default 50, validated to `1..=200`
  rather than silently clamped. When the response reaches the requested limit,
  `dsc` writes a stderr warning that older matching entries may exist.

This first cut makes one request only; it does not paginate through the full
history. A future pagination spec must capture the live endpoint's paging
contract before adding `--all` or client-side substring filtering. Fleet-wide
aggregation is `R20`'s job, not this command's.

## Output

- **text** (default): one line per entry - timestamp, action name, acting
  user, target user (`-` when absent), subject.
- **json** / **yaml**: the full entry list, one object per row: `id`,
  `action_name`, `acting_user` (`{id, username}` or absent), `target_user`
  (ditto), `subject`, `details`, `previous_value`, `new_value`, `created_at`.
  `previous_value` and `new_value` preserve either scalar strings or structured
  JSON values as returned by Discourse. Field set matches Discourse's
  `UserHistorySerializer` minus a few
  presentation-only fields (`ip_address`, `email`, `context`, `custom_type`)
  not needed for the audit-trail use case.

## Notes

- Requires an admin API key - `/admin/logs/*` is an admin-only route.
- Read-only: ignores the global `--dry-run` flag like other inspection
  commands (`user activity`, `search`, `analytics`).
- Not yet covered: custom/plugin action types, `--action-id` (the numeric
  code, vs. the validated symbolic `--action` name), pagination, substring
  subject search, and cross-forum aggregation - add on demand.
