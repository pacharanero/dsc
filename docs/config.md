# dsc config

Inspect and validate your `dsc.toml` configuration.

Running `dsc config` without a subcommand prints the active config file path and the full search order, showing which candidates exist on disk:

```text
$ dsc config
Active config: /home/marcus/.config/dsc/dsc.toml

Search order:
  1. dsc.toml
  2. /home/marcus/.config/dsc/dsc.toml <-- active
  3. /etc/xdg/dsc/dsc.toml
  4. /etc/dsc/dsc.toml
  5. /etc/dsc.toml
  6. /usr/local/etc/dsc.toml
```

## dsc config check

```text
dsc config check [--format text|json|yaml] [--skip-ssh]
```

Probes each configured Discourse and reports two things per install:

- **API** — sends `GET /about.json` with the configured `apikey`/`api_username`. Reports `ok` on 2xx, flags 401/403 with a hint to check credentials, and surfaces other HTTP errors verbatim.
- **SSH** (only when `ssh_host` is set on the entry, and `--skip-ssh` is not passed) — runs `ssh -o BatchMode=yes -o ConnectTimeout=5 <host> true`. Reports the first stderr line on failure so problems are diagnosable at a glance.

Exits non-zero if any install fails any check, making it suitable for CI or pre-deploy gates.

Examples:

```bash
dsc config check
dsc config check --format json
dsc config check --skip-ssh       # API-only check, much faster
```
