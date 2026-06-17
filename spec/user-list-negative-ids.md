# `dsc user list` - tolerate negative user IDs (Discourse system accounts)

> **Status: Fixed in v0.10.14.** `UserSummary.id`, `UserDetail.id`, and every
> user-action helper signature widened from `u64` to `i64`. Regression tests
> in `src/api/users.rs` cover both `system` (-1) and `discobot` (-2).
> JSON/YAML output for normal users is unchanged; system accounts that
> previously broke parsing now appear in the listing.

Spec/bug for `dsc user list`. Goal: make `dsc user ls <discourse>` succeed on every page even when the page contains Discourse's built-in system accounts. Driver: extracting the full member email list from the `rjc-vcop` forum (restorativejustculture.org) for an NHS Trust analysis.

## Motivation

I was pulling the complete user list from a real Discourse install with `dsc user ls rjc-vcop -f json -p <n>`. Pages 1-3 parsed fine, but page 4 and page 5 both failed with:

```
Error: parsing user list response

Caused by:
    invalid value: integer `-2`, expected u64 at line 1 column 58283
```

The forum has 418 active users across 5 pages, so the failure made it impossible to retrieve roughly the last ~120 users (and any other listing/page that happens to include a system account) through `dsc`. I worked around it by calling the admin endpoint directly with `curl` and parsing the JSON myself - see the reference section.

## Current state (as of 2026-06-17)

`dsc user list` (alias `ls`) calls `/admin/users/list/<type>.json?show_emails=true&page=<n>` and deserialises each row into `UserSummary` (`src/api/users.rs`). The same `id: u64` typing is used for `UserDetail`.

```rust
// src/api/users.rs
pub struct UserSummary {
    pub id: u64,
    ...
}

pub struct UserDetail {
    pub id: u64,
    ...
}
```

Discourse's built-in system accounts use **negative** IDs:

- `system` → `id: -1`
- `discobot` → `id: -2`

These accounts are returned by the `active` listing (here `discobot` lands on page 4 and `system` on page 5). Because `id` is `u64`, serde rejects the negative value and the whole page fails to parse. `dsc version` = `0.10.9`.

The error is not user-specific to this forum - any Discourse install will surface `system`/`discobot` in the relevant listing page, so this is reproducible anywhere.

### Reproduction

```text
$ dsc user ls rjc-vcop -f json -p 4
Error: parsing user list response

Caused by:
    invalid value: integer `-2`, expected u64 at line 1 column 58283
```

(`-2` is `discobot`; on page 5 the same error reports `-1` for `system`.)

## Proposed fix

Change the deserialised `id` field from `u64` to `i64` in both `UserSummary` and `UserDetail`, so rows for system accounts parse. Discourse IDs are signed on the wire, and negative IDs are reserved for system accounts.

Knock-on: the admin action helpers in `src/api/users.rs` (`suspend_user`, `unsuspend_user`, `silence_user`, `unsilence_user`, `grant_admin`, `revoke_admin`, `grant_moderation`, `revoke_moderation`, and the shared `put_admin_user_action`) take `user_id: u64`. Either:

- widen those signatures to `i64` (simplest, mirrors the wire type), or
- keep them `u64` and cast at the call site in `src/commands/user.rs`.

Widening to `i64` is recommended - it keeps one type for "a Discourse user id" end to end. Acting on a system account is a no-op the API will reject anyway, so no extra guard is required; optionally, the action commands could refuse `id < 0` with a friendly message ("cannot moderate the built-in system account").

A narrower alternative - a custom deserializer that maps the `id` field only - was considered but adds more code than simply using the correct signed type.

## Reference: API call observed in the field

Request (Discourse admin API; tested against restorativejustculture.org, Discourse hosted/stable as of 2026-06-17):

```text
GET /admin/users/list/active.json?show_emails=true&page=4
Api-Key: <REDACTED>
Api-Username: marcusbaw
```

Relevant row from the response (the one that trips the parser):

```json
{
  "id": -2,
  "username": "discobot",
  "name": "discobot",
  "email": "no_email",
  "trust_level": 4,
  "admin": false,
  "moderator": false
}
```

Page 5 contains the equivalent `system` row with `"id": -1`.

## Backward compatibility

Changing `id` from `u64` to `i64` is source-compatible for all current internal uses (the value is only printed and passed to action helpers). The JSON/YAML output format is unchanged for normal users; system accounts that previously caused a hard error will now appear in the output. If any downstream tooling assumed `dsc user ls` never emits negative IDs, it will now see `-1`/`-2` - this is the correct behaviour but worth a changelog note.

## Out of scope

- Filtering system accounts out of the listing by default (they are legitimately part of `active`; callers can filter on `id < 0` or username if they want).
- Any change to which listing types are exposed or to pagination.
- Validation/guards on moderating system accounts (optional nicety noted above, not required for the fix).

## Audit of downstream code (post-implementation, 2026-06-17)

After widening to `i64`, the full set of code paths that touch a Discourse user ID was audited. **No other code requires changes**:

| Code path | Verdict |
|---|---|
| `UserSummary.id` / `UserDetail.id` printing | Safe - just `{}` formatting; `-1` renders fine in text/JSON/YAML |
| `suspend_user`, `unsuspend_user`, `silence_user`, `unsilence_user`, `grant_admin`, `revoke_admin`, `grant_moderation`, `revoke_moderation` | Safe - all widened to `i64`, all pass through to URL building. Acting on a system account hits `PUT /admin/users/-1/suspend.json` and Discourse responds 4xx; the existing `http_error` path surfaces it as a normal command error (raw HTTP message, not a `dsc`-specific friendly one - see below) |
| `create_user` return | Safe - Discourse only ever assigns positive IDs to newly-created accounts |
| `dsc user info <username>` | Safe - resolves by username; printed id can now be `-1`/`-2` for system rows |
| `dsc user activity <username>` | Safe - operates on the `username` string, not a user id |
| `dsc user groups list/add/remove` | Safe - operates on usernames |
| `dsc invite send/bulk` | Safe - no user IDs involved |
| `UserAction` struct (used by `dsc user activity`) | Safe - carries `topic_id`/`post_id`/`post_number` only, never a user id |
| Integration tests in `tests/` | Safe - none touch the user-list path |

### Friendly-error UX (still deliberately out of scope, just documented)

`dsc user suspend system` resolves the username, then calls the API with `id: -1`. The result is the standard Discourse 4xx surfaced via the existing `http_error` path - not a `dsc`-specific message like "cannot moderate the built-in system account". This matches the original spec intent (no extra guard required) but is worth noting so the behaviour isn't a surprise.

### Forward-looking notes for unrelated work

These are flagged because they will touch this area in future, not because they need doing now:

- **Analytics per-user-walk metrics.** The stubbed metrics in [spec/analytics.md](analytics.md) (`new_contributors`, `reactivated_users`, `lost_regulars`, `unique_posters`, `top_10_share`, `returning_poster_rate`) will eventually walk `admin_list_users` and compute things like "users who posted ≥ N times". When that work lands, those derivations **must filter out system accounts** (e.g. `.filter(|u| u.id > 0)` at the right point), otherwise `system` and `discobot` would be counted as "lost regulars" simply because they don't post. Easy one-liner; just don't forget.
- **Future "pull/push for users" surfaces** (not currently planned). If any future snapshot/restore workflow round-trips user data, it would need to skip system rows on push (Discourse won't let you re-create them).
