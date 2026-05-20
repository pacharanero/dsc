# dsc update

Runs remote OS and Discourse update workflows over SSH.

```
dsc update <name|all> [--no-changelog] [--yes] [--parallel] [--max <n>]
```

## Flags

- `--no-changelog` — skip changelog posting.
- `--yes` (or `-y`) — auto-confirm the changelog post prompt (non-interactive mode).
- `--parallel` (or `-p`) — enable concurrent workers (only with `all`).
- `--max <n>` (or `-m <n>`) — set worker count when `--parallel` is enabled (default: `3`).

## Update workflow

1. OS package update over SSH.
2. Reboot (if applicable).
3. Check whether the running Discourse commit matches the latest `stable` branch commit on GitHub. If they match, skip the rebuild.
4. Discourse rebuild (`./launcher rebuild app`) — only if step 3 found a newer commit available.
5. Cleanup (`docker container prune -f && docker image prune -f`).
6. Fetch version info from the homepage `<meta name="generator" ...>` tag.
7. Optionally post a changelog checklist to the configured topic.

If the OS update command fails, `dsc update` aborts after attempting the rollback command (when configured).

## Changelog template

The changelog is posted as a checklist to the topic specified by `changelog_topic_id`:

```md
- [x] OS updated {{ubuntu_os_version}}
- {% if rebooted %} {{[x] Server rebooted}} {% endif %}
- [x] Updated Discourse:
  - Initial version: {{ before_version }} [{{ before_commit_hash | truncate 7 }}](https://github.com/discourse/discourse/commit/{{ before_commit_hash }})
  - Updated version: {{ after_version }} [{{ after_commit_hash | truncate 7 }}](https://github.com/discourse/discourse/commit/{{ after_commit_hash }})
- [x] `./launcher cleanup` Total reclaimed space: {{ reclaimed_space }}
- [x] Root disk usage (df -h /): {{ root_disk_usage }}
```

## Parallel updates

```bash
# Update all installs, 4 at a time, non-interactively
dsc update all --parallel --max 4 --yes
```

In sequential mode (without `--parallel`), updates run one-by-one. `all` is a reserved name for `dsc update all`.

## Skipping behaviour

- **No `ssh_host`:** `dsc update all` skips any Discourse instance that has no `ssh_host` configured. These are typically read-only references (e.g. Discourse Meta) or instances not managed via SSH.
- **Already up to date:** Before running the rebuild, `dsc update` queries the GitHub API for the latest commit on the `discourse/discourse` `stable` branch and compares it with the commit reported by the running instance. If they match, the rebuild is skipped (OS updates still run). If GitHub is unreachable or the running commit is unknown, the rebuild proceeds as normal (fail-open).

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DSC_SSH_OS_UPDATE_CMD` | `sudo -n DEBIAN_FRONTEND=noninteractive apt update && sudo -n DEBIAN_FRONTEND=noninteractive apt upgrade -y` | OS update command. |
| `DSC_SSH_OS_UPDATE_ROLLBACK_CMD` | *(none)* | Rollback command if OS update fails. |
| `DSC_SSH_REBOOT_CMD` | `sudo -n reboot` | Reboot command. |
| `DSC_SSH_OS_VERSION_CMD` | `lsb_release -d \| cut -f2` | OS version detection (fallback: `/etc/os-release`). |
| `DSC_SSH_UPDATE_CMD` | `cd /var/discourse && sudo -n ./launcher rebuild app` | Discourse rebuild command. |
| `DSC_SSH_CLEANUP_CMD` | `cd /var/discourse && sudo -n ./launcher cleanup` | Post-rebuild cleanup command. |
| `DSC_SSH_STRICT_HOST_KEY_CHECKING` | `accept-new` | SSH host key checking mode (set empty to omit). |
| `DSC_SSH_OPTIONS` | *(none)* | Extra SSH options (space-delimited). |
| `DSC_DISCOURSE_BOOT_WAIT_SECS` | `15` | Seconds to wait after rebuild before fetching `about.json`. |
| `DSC_COLOR` | `auto` | ANSI color output (`auto`/`always`/`never`). `NO_COLOR` also disables color. |
