# dsc log

View admin audit trails.

## dsc log staff

```text
dsc log staff <discourse> [--action <name>] [--acting-user <username>] [--target-user <username>] [--subject <text>] [--since <dur>] [--limit <n>] [--format text|json|yaml]
```

Lists entries from the staff action log — the same audit trail behind `/admin/logs/staff-action-logs` in the web UI: setting changes, suspensions, deletions, admin grants, and the rest of Discourse's `UserHistory` records.

All filters are optional and combine (AND):

- `--action` — symbolic action name, e.g. `change_site_setting`, `suspend_user`, `delete_post`, `grant_admin`.
- `--acting-user` — the staff member who performed the action.
- `--target-user` — the user the action was performed on.
- `--subject` — substring match on the subject field (often a setting name).
- `--since` — relative duration (`7d`, `24h`) or an ISO-8601 date/timestamp; translated to Discourse's `start_date` filter.
- `--limit` — rows to fetch, default 50; Discourse caps this server-side at 200.

```bash
# Recent activity, most recent first
dsc log staff myforum

# Who changed a setting, and to what
dsc log staff myforum --action change_site_setting --subject login_required

# One admin's actions in the last week, as JSON
dsc log staff myforum --acting-user alice --since 7d -f json
```

## Notes

- Requires an admin API key — this is an admin-only endpoint.
- Read-only: `dsc log staff` never writes anything, and ignores `--dry-run`.
