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

- `--action` - symbolic action name (`action_name` server-side), e.g.
  `change_site_setting`, `suspend_user`, `delete_post`, `grant_admin`. Not
  validated client-side - Discourse's numeric `Actions` table isn't exposed
  over the API, so unknown names just yield zero rows rather than an error.
- `--acting-user` - the staff member who performed the action.
- `--target-user` - the user the action was performed on (absent for
  non-user-targeted actions, e.g. a setting change).
- `--subject` - substring match on the subject field (often a setting name).
- `--since` - relative duration (`7d`, `24h`) or an ISO-8601 date/timestamp,
  parsed with the same `parse_since_cutoff` used by `analytics` and
  `update log`, then sent as Discourse's `start_date` (day granularity - the
  endpoint doesn't support finer-grained server-side filtering).
- `--limit` - rows to fetch, default 50. Discourse caps `/admin/logs/staff_action_logs.json`
  at 200 server-side regardless of what's requested; `dsc` clamps to that
  before sending so a large `--limit` doesn't silently under-deliver without
  explanation.

No pagination beyond `--limit`/`--since` in this first cut - fleet-wide
aggregation is `R20`'s job, not this command's.

## Output

- **text** (default): one line per entry - timestamp, action name, acting
  user, target user (`-` when absent), subject.
- **json** / **yaml**: the full entry list, one object per row: `id`,
  `action_name`, `acting_user` (`{id, username}` or absent), `target_user`
  (ditto), `subject`, `details`, `previous_value`, `new_value`, `created_at`.
  Field set matches Discourse's `UserHistorySerializer` minus a few
  presentation-only fields (`ip_address`, `email`, `context`, `custom_type`)
  not needed for the audit-trail use case.

## Notes

- Requires an admin API key - `/admin/logs/*` is an admin-only route.
- Read-only: ignores the global `--dry-run` flag like other inspection
  commands (`user activity`, `search`, `analytics`).
- Not yet covered: `--action-id` (the numeric code, vs. the symbolic
  `--action` name) and cross-forum aggregation - add on demand.
