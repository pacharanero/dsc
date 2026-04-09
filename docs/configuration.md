# Configuration

If `--config <path>` is not provided, `dsc` searches for a config in this order:

1. `./dsc.toml`
2. `$XDG_CONFIG_HOME/dsc/dsc.toml` (or `~/.config/dsc/dsc.toml` when `XDG_CONFIG_HOME` is unset)
3. System config locations (`$XDG_CONFIG_DIRS` entries as `<dir>/dsc/dsc.toml`, then `/etc/xdg/dsc/dsc.toml`, `/etc/dsc/dsc.toml`, `/etc/dsc.toml`, `/usr/local/etc/dsc.toml`)

If none are found, it defaults to `./dsc.toml` (created on first write command).

Each Discourse instance lives under a `[[discourse]]` table. See [dsc.example.toml](../dsc.example.toml) for a fuller template. Minimum useful fields are `name`, `baseurl`, `apikey`, and `api_username`.

```toml
[[discourse]]
name = "myforum"
fullname = "My Forum"
baseurl = "https://forum.example.com"
apikey = "your_api_key_here"
api_username = "system"
changelog_topic_id = 123
ssh_host = "forum.example.com"
tags = ["production", "client-a"]
```

## Field reference

| Field | Required | Description |
|---|---|---|
| `name` | yes | Short slugified identifier (no spaces). |
| `baseurl` | yes | Forum URL, no trailing slash. |
| `apikey` | for API commands | Discourse API key. |
| `api_username` | for API commands | User to act as (usually `system`). |
| `fullname` | no | Display name / site title. Auto-populated by `dsc add` and `dsc import` when fetchable. |
| `ssh_host` | for `update` | SSH config host name for remote updates. |
| `changelog_topic_id` | for changelog | Topic ID for update changelog posts. |
| `tags` | no | Labels for organising installs; used with `--tags` filtering. |
| `enabled` | no | Defaults to `true`. Set `false` to skip in bulk operations. |

## Notes

- `dsc add` without `--interactive` appends a full `[[discourse]]` template containing every supported config key, using placeholders like `""`, `[]`, and `0`.
- Empty strings and `0` values are treated as "unset" (most commands behave as if the key is missing).
- Most forum read/write commands require `apikey` and `api_username`. If they are missing, `dsc` will fail with a clear message.
- SSH credentials are not stored in `dsc.toml`; set up SSH keys and use an SSH config file.
