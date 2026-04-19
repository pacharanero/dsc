# dsc pm

Send and list private messages.

## dsc pm send

```text
dsc pm send <discourse> <recipients> --title <text> [<local-path>]
```

Sends a PM. `<recipients>` is comma-separated (semicolon also accepted) — usernames or group names. The body is read from `<local-path>` if given, otherwise stdin (or `-`). Honours `--dry-run`.

```bash
# To one user from a file
dsc pm send myforum alice -t "Welcome" ./welcome.md

# To several recipients via stdin
echo "Sprint review notes attached." | dsc pm send myforum "alice,bob,@dev-team" -t "Sprint #42"

# Dry-run
dsc -n pm send myforum alice -t "Test" ./body.md
```

## dsc pm list

```text
dsc pm list <discourse> <username> [--direction inbox|sent|archive|unread|new] [--format text|json|yaml]
```

Lists PM topics for a user in the given direction (default `inbox`). Default text mode prints id, title, last activity timestamp, and the username of the last poster.

```bash
dsc pm list myforum alice
dsc pm list myforum alice --direction sent --format json
dsc pm list myforum alice -d unread
```

## Notes

- The API user (the one whose key is in `dsc.toml`) needs admin scope to list arbitrary users' PMs. Without admin, you can only list your own.
- A PM is a topic with `archetype=private_message`; `dsc topic reply <topic-id>` works on it just like any other topic for follow-ups.
