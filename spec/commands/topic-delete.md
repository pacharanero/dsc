# `dsc topic delete` - delete, purge, and restore topics by topic ID

> **Status: implemented (unreleased).** `dsc topic delete` / `rm` soft-deletes one or more topics by topic ID, honours global `--dry-run`, and supports `--purge` / `--permanent` for permanent deletion. `dsc topic restore` restores a soft-deleted topic via `PUT /t/{id}/recover`. `dsc topic list --deleted [query]` discovers restorable topic IDs via Discourse search (`status:deleted`).

Spec for deleting a topic by its topic ID. Driver: real-world use archiving house-property records out of a private Discourse - two topics were pulled locally with `dsc topic pull --full` and then needed to be deleted from the forum.

## Motivation

`dsc topic pull <discourse> <topic_id> --full` is the documented way to archive a topic to local Markdown. The natural follow-on - deleting the now-archived topic from the forum - had no `dsc` command. `dsc post delete` exists but requires a post ID, and `dsc topic pull --full` frontmatter records only `topic_id`, `url`, `posts_count`, and `pulled_at` - not the underlying post IDs. The agent had to fall back to raw `curl` with headers parsed out of `dsc.toml`.

This is the symmetric counterpart to `dsc topic new` (create) and is a standard part of any pull-then-cleanup archive workflow.

## Implemented CLI surface

```text
dsc topic delete <discourse> <topic_id> [<topic_id>...] [--purge]
dsc topic rm     <discourse> <topic_id> [<topic_id>...] [--purge]
dsc topic restore <discourse> <topic_id>
dsc topic list <discourse> --deleted [query] [--format text|json|yaml]
```

- **`dsc topic delete` / `rm`** - deletes whole topics via `DELETE /t/{id}.json`. By default this is a soft-delete.
- **Batch deletion** - accepts multiple topic IDs and prints one line per topic.
- **`--dry-run`** - fetches each topic first and prints the title/post count plus the planned delete; sends nothing.
- **`--purge` / `--permanent`** - sends `DELETE /t/{id}.json?permanent=true` for irreversible deletion.
- **`dsc topic restore`** - recovers a soft-deleted topic via `PUT /t/{id}/recover.json`. This is useful enough to ship with delete: soft-delete is the default, so a first-class undo path belongs next to it.
- **`dsc topic list --deleted [query]`** - discovers deleted topic IDs for restore. It uses Discourse search with `status:deleted`; optional `query` terms narrow the result set. General topic search remains `dsc search`.

## Reference: API calls observed in the field

Tested against bawmedical.co.uk (Discourse 2026.7.0-latest), topics 1178 and 969.

```text
DELETE /t/1178
Api-Key: <redacted>
Api-Username: marcusbaw

→ 200 OK
(empty body)
```

```text
DELETE /t/969
Api-Key: <redacted>
Api-Username: marcusbaw

→ 200 OK
(empty body)
```

Notes:

- The endpoint is `DELETE /t/{id}` (no `.json` suffix needed; Discourse accepts both). `dsc` uses `.json` for consistency.
- Auth is via `Api-Key` / `Api-Username` headers (the same pair `dsc` already uses for all admin API calls, read from `dsc.toml`).
- A successful deletion returns 200 with an empty body - no JSON to parse.
- A 404 returns `{"errors":["The requested URL or resource could not be found."],"error_type":"not_found",...}`.
- A 403 returns `["BAD CSRF"]` when the key is passed as query params (`?api_key=...&api_username=...`) rather than headers. The header form works; `dsc` already uses headers internally so this is just a note for anyone hand-testing with `curl`.

### Restore and deleted-topic discovery

Confirmed from Discourse routes (`config/routes.rb`, main branch):

```text
PUT /t/:topic_id/recover  → topics#recover
```

`dsc` calls:

```text
PUT /t/{topic_id}/recover.json
```

For discovery, there is no separate topic-trash list endpoint in the public route surface. The most natural user-facing command is therefore a small wrapper around Discourse search:

```text
GET /search.json?q=status%3Adeleted
GET /search.json?q=<query>+status%3Adeleted
```

This keeps the mental model simple: `topic list --deleted` answers "what IDs can I restore?", while `dsc search` remains the general search command.

### Related gap: post IDs in `--full` output

While not required for this spec (topic deletion uses the topic ID directly), the lack of post IDs in `dsc topic pull --full` frontmatter remains a separate possible improvement. Adding `post_ids: [1111, 1112, ...]` to the YAML frontmatter would let `dsc post delete` target individual posts in a pulled thread without an extra API call. That belongs with [topic-pull-full-thread.md](topic-pull-full-thread.md), not here.

## Phases

### Phase 1 - blocking

- [x] Add `delete` (alias `rm`) subcommand to `dsc topic` taking `<discourse> <topic_id>`.
- [x] `--dry-run` prints topic title + post count + planned delete.
- [x] Calls `DELETE /t/{id}` via the existing `DiscourseClient`, surfaces 404/403 errors through the shared HTTP error path.

### Phase 2 - iteration ergonomics

- [x] `--purge` / `--permanent` for permanent deletion (`permanent=true`).
- [x] Batch deletion: `dsc topic delete <discourse> <id1> <id2> ...`.

### Restore/discovery follow-up shipped with the same work

- [x] `dsc topic restore <discourse> <topic_id>` via `PUT /t/{id}/recover.json`.
- [x] `dsc topic list <discourse> --deleted [query]` to discover restorable topic IDs.

## Backward compatibility

Purely additive. No existing `dsc topic` behaviour changes. The `delete`/`rm` alias pattern mirrors `dsc post delete` / `dsc post rm`.

## Out of scope

- **Deleting individual posts in a topic** - already covered by `dsc post delete`; the only gap is surfacing post IDs in `--full` output, tracked in [topic-pull-full-thread.md](topic-pull-full-thread.md) Phase 2.
- **A full general-purpose topic list command** - `topic list --deleted` is intentionally scoped to deletion recovery. General discovery stays under `dsc search`.
- **Trash status details beyond search results** - if operators need deletion timestamps, deleter username, or audit-log integration, file a separate field spec for staff-action-log support.
