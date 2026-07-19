# dsc notification

List and mark read the API user's own Discourse notifications.

`dsc` acts as the account behind the configured `api_username`, so these are that account's notifications — typically the forum's admin/bot account, not an arbitrary user's.

## dsc notification list

```text
dsc notification list <discourse> [--filter read|unread] [--type <names>] [--limit <n>] [--format text|json|yaml]
```

Lists notifications newest first.

- `--filter` — only `read` or `unread` notifications.
- `--type` — only notifications of these comma-separated built-in Discourse notification type names, e.g. `liked,mentioned,private_message`. Unknown and custom/plugin names are rejected rather than risking an unfiltered query.
- `--limit` — newest-first rows to fetch, default 30; must be from 1 through 60 (Discourse's own maximum). If exactly this many are returned, `dsc` warns on stderr that older matching notifications may exist.

```bash
# Recent notifications, most recent first
dsc notification list myforum

# Unread likes and mentions only, as JSON
dsc notification list myforum --filter unread --type liked,mentioned -f json
```

## dsc notification read

```text
dsc notification read <discourse> (--id <id> | --type <names> | --all)
```

Marks notification(s) read. Exactly one selector is required:

- `--id` — mark this single notification ID read.
- `--type` — mark every unread notification of these comma-separated built-in Discourse notification type names read.
- `--all` — mark every unread notification read.

Honours `--dry-run`.

```bash
# Mark one notification read
dsc notification read myforum --id 12345

# Clear every unread like/mention
dsc notification read myforum --type liked,mentioned

# Clear the whole inbox
dsc notification read myforum --all
```

## Notes

- Uses the standard user-facing `/notifications.json` endpoint, so it works with any valid API key/username pair — no admin scope required.
- `--type` and `notification_type` in JSON/YAML output use Discourse's built-in `Notification.types` names; custom/plugin notification types are not supported yet and render as their raw numeric ID.
- This first version reads one page only; it does not yet paginate through older notifications.
