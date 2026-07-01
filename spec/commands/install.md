# `dsc install` and `dsc harden` stage 3 - provisioning spec

Spec for the unstarted `dsc install` command and the not-yet-shipped finishing items in `dsc harden` stage 3. Together these complete the from-zero Discourse bootstrap story: a fresh Ubuntu IP becomes a hardened box becomes a running Discourse with an entry in your `dsc.toml`.

## Motivation

`dsc update` assumes an already-running Discourse. `dsc harden` (stages 1+2 shipped) prepares a fresh server for one. The gap is the middle: declaratively installing Discourse on a hardened box. Today this is done by hand (clone `discourse_docker`, edit `app.yml`, `launcher bootstrap`, `launcher start`, then add a `[[discourse]]` block to `dsc.toml` manually). This spec closes that gap.

## Current state (as of 2026-06-09)

- `dsc harden` ships stages 1 + 2 (preflight, non-root sudo user, key-only sshd on a non-default port, self-lockout guard). Stage 3 (timezone, swap, journald, unattended upgrades, fail2ban, rootless Docker, ufw) has its config keys wired in [src/commands/harden.rs](../../src/commands/harden.rs) `HardenOpts` but the SSH-side execution and tests are pending.
- `dsc install` does not exist.
- `DiscourseConfig` lacks `ssh_user`/`ssh_port` fields. Today `dsc update` finds the SSH host by name only (relies on `~/.ssh/config`); `dsc install` needs to write these on success.

## `dsc harden` stage 3 - finishing items

In rough execution order. All gated behind the existing config keys; the work is the SSH-side execution path.

1. **Timezone + time sync.** `timedatectl set-timezone <timezone>` (default `UTC`), then verify `timedatectl status` shows synchronised. Install `chrony` if `systemd-timesyncd` is unavailable.
2. **Swap file.** Check `swapon --show`; if no swap, create `/swapfile` of `swap_size_gb` (default 2 GB), `mkswap` + `swapon`, persist in `/etc/fstab`, set `vm.swappiness=10` via `/etc/sysctl.d/99-dsc.conf`.
3. **Journald cap.** Write `SystemMaxUse=<journald_max_use>` (default `500M`) to `/etc/systemd/journald.conf.d/size-cap.conf`, `systemctl restart systemd-journald`.
4. **Unattended security upgrades.** Ensure `unattended-upgrades` is installed; write `/etc/apt/apt.conf.d/20auto-upgrades` with both `Update-Package-Lists` and `Unattended-Upgrade` set to `1`.
5. **fail2ban.** `apt install fail2ban`; minimal jail for sshd on the new port.
6. **Rootless Docker** (when `docker_rootless = true`, which is the default): `curl -fsSL https://get.docker.com | sh`, then `apt install uidmap`, then as the new user `dockerd-rootless-setuptool.sh install`, then `sudo setcap cap_net_bind_service=ep $(which rootlesskit)` so Discourse can bind 80/443, then `systemctl --user restart docker`, then `loginctl enable-linger <new_user>` so the user-level systemd units survive logout.
7. **`ufw`.** Allow 22, `<ssh_port>`, 25, 80, 443; allow 60000:61000/udp when `--mosh` flag is present (CLI flag still TODO). Apply each `extra_ufw_allow` entry. `ufw --force enable`.

### Gotchas to remember

- **sshd port change + cloud firewall.** Hetzner / Digital Ocean / etc. have their own firewall layer that `dsc` can't reach. The new SSH port needs opening *there too*. Document in stdout near the end of harden output: "now open port `<ssh_port>` in your cloud provider's firewall".
- **Rootless Docker + privileged ports.** The `setcap cap_net_bind_service=ep` step on `rootlesskit` is non-optional for Discourse to bind 80/443. Easy to forget; bake into the harden output, not just the docs.
- **`loginctl enable-linger`.** Without it, user-level systemd units (rootless Docker daemon, the Discourse container) die on SSH disconnect. Same status as the setcap line - non-optional.
- **MOSH ports.** Only opened when the operator asks for them. Add a `--mosh` flag at the same time as the rest of stage 3 wiring.
- **Ubuntu version drift.** Stage 3 should test the OS detection on whatever ships at the time. Today the harden code accepts `ID=ubuntu`; `discourse_docker` may not have a base image for the very newest LTS yet (2-3 month lag historically). The pragmatic answer is "try the previous LTS first if you hit this".

## `dsc install` - new command

Templated `app.yml` + `launcher bootstrap + start` + `dsc.toml` write, all over SSH.

### CLI surface

```text
dsc install <name> --host <host>
                   [--email admin@example.com]
                   [--smtp-host …] [--smtp-port …] [--smtp-user …] [--smtp-pass-stdin]
                   [--image base]
                   [--branch tests-passed]
                   [--ssh-user discourse] [--ssh-port 2227]
                   [--bootstrap-admin]              # see "first admin flow" below
                   [--dry-run]
```

### What it does

1. **Connect** as `<ssh-user>@<host>:<ssh-port>` (defaulting to whatever `dsc harden` produced - read these from a transient state file or accept on the CLI).
2. **Memory preflight.** `free -m`; bail if < 1024 MB, warn at < 2048 MB. Discourse will OOM in production below 2 GB without aggressive swap/tuning.
3. **Clone `discourse_docker`** to `/var/discourse` if not present. Otherwise `git pull`.
4. **Render `app.yml`** locally from a template - string-substitute `hostname`, `developer_emails`, SMTP host/port/user/pass, `image`/`branch`. No interactive `discourse-setup`. Heredoc or scp into `containers/app.yml` on the remote.
5. **`./launcher bootstrap app && ./launcher start app`** (with sudo, or as the rootless-Docker user with `--user-mode rootless` whatever the equivalent is - check during implementation).
6. **Poll `https://<host>/about.json`** until 200 (60s timeout). Fail with a clear error if it never comes up.
7. **Append a `[[discourse]]` entry** to `dsc.toml` via the existing `save_config` path: `name`, `baseurl: https://<host>`, `ssh_host: <host>`, `ssh_user: <ssh-user>`, `ssh_port: <ssh-port>`. Leave `apikey` and `api_username` empty.
8. **Print next-steps footer** with the web-UI admin signup URL and how to mint and add the API key (unless `--bootstrap-admin` did this for you).

### First admin flow

Two modes:

- **Default (manual).** User opens `https://<host>/admin` in a browser, signs up the first admin, generates an admin API key in the Admin UI, pastes it into `dsc.toml`. Predictable, no `rails runner` required.
- **`--bootstrap-admin` (later).** `docker exec app rails runner …` to create the admin and mint the API key in one shot, populating `dsc.toml` with both fields. Roadmapped as a follow-up - safer to ship the manual mode first.

### Honours `--dry-run`

Yes. Print the rendered `app.yml`, the launcher commands, and the `[[discourse]]` block that would be appended to `dsc.toml`. Do not connect over SSH for write operations.

## Config schema additions

Add to `DiscourseConfig` in [src/config.rs](../../src/config.rs):

```rust
#[serde(default, deserialize_with = "deserialize_opt_string_empty_as_none")]
pub ssh_user: Option<String>,
#[serde(default, deserialize_with = "deserialize_opt_u64_zero_as_none")]
pub ssh_port: Option<u64>,
```

`dsc update` already reads `ssh_host`; once `ssh_user`/`ssh_port` are available it should consume them too (today it relies entirely on `~/.ssh/config`).

## Tests

- Stage 3 individual steps tested in isolation (mock SSH session, assert generated config files match golden snapshots).
- `dsc install` rendering the templated `app.yml` from flags is unit-testable without any SSH; the SSH side gets integration-tested against a fresh Hetzner VM in the manual release rehearsal.
- Memory-preflight bail / warn at the documented thresholds.
- `--dry-run` round-trips: prints, never connects.

## Out of scope

- Provisioning the cloud VM itself (`hcloud server create`, `doctl compute droplet create`, etc.). Out of `dsc`'s lane.
- `discourse-setup` interactive wizard parity. The whole point is to not need it.
- Multi-container Discourse setups (separate `data.yml` + `web_only.yml`). Single-container `app.yml` only for v1.
- Discourse downgrades. `--branch tests-passed` and `--image base` are the safe path; if a user wants to pin a specific commit they can edit `app.yml` after install.

## Phasing

### Phase 1 - finish `dsc harden` stage 3

Blocking for `dsc install` (no point installing Discourse on an unhardened box).

### Phase 2 - `dsc install` minimum viable path

`--host`, `--email`, SMTP flags, manual-admin mode. Single-container `app.yml`.

### Phase 3 - `--bootstrap-admin`

`rails runner` admin creation + API-key minting. Risky (one-shot, hard to rerun) so save for after Phase 2 has soaked.

### Phase 4 - polish

Memory preflight refinements, parallel installs (`dsc install all` from a multi-name flag set), more SMTP providers' presets, `app.yml` template variations.
