# `dsc tag` — declarative pull/push spec

Spec for handoff to the agent that maintains `dsc`. Goal: manage a Discourse instance's full tag taxonomy (tags **and** tag groups) as a single version-controlled file, mirroring the pull/push pattern `dsc` already uses for `category`, `palette`, and `theme`.

Target workflow:

```
dsc tag pull <discourse> [path]    # write server taxonomy → file
# edit + commit the file
dsc tag push <discourse> <path>    # apply file → server
```

## Current state (as observed 2026-05-25)

`dsc tag` exposes only `list`, `apply`, `remove` (all topic-level). `dsc tag list` already supports `-f text|json|yaml`. There is **no** pull/push for tags and **no** tag-group support anywhere in `dsc`. Both need adding.

## New commands

### `dsc tag pull <DISCOURSE> [LOCAL_PATH]`

- Serialises **all** tags and tag groups to one file (the whole taxonomy is the unit of version control — unlike `topic`/`category` which are multi-file, tags are few and interdependent, so a single document is correct).
- Default `LOCAL_PATH`: `tags.yaml`.
- Format inferred from extension: `.yaml`/`.yml` (default) or `.json`. Reuse the serializers already behind `tag list -f`.
- Emit **definitions only** — exclude usage counts and any server-derived/read-only fields, so repeated pulls stay diff-clean.

### `dsc tag push <DISCOURSE> <LOCAL_PATH>`

- Reads the file and reconciles server state toward it.
- **Default semantics: upsert** — create missing tags/groups, update changed ones, never delete.
- `--prune`: additionally delete tags and tag groups present on the server but absent from the file. Off by default.
- Must honour the global `-n/--dry-run`: print the plan (per-tag and per-group create/update/delete) without sending writes. This is the primary safety mechanism, consistent with the rest of `dsc`.
- Idempotent: a push with no file change must be a no-op (compare normalised values; no spurious PUTs).

## File schema (the contract)

`tags.yaml` in this repo is a populated example. `pull` must emit this shape; `push` must accept it.

```yaml
version: 1

tags:              # optional; per-tag metadata only. A tag named only inside a
  - name: covers   #   group still exists — these entries just attach a description.
    description: ...

tag_groups:
  - name: Role
    description: ...        # optional
    one_per_topic: false    # default false
    parent_tag: null        # optional; a tag that must be present for the group to apply
    permissions:            # optional; default = everyone may use. Map of group → level.
      everyone: full        #   levels mirror Discourse tag-group perms: full | readonly
    tags:
      - guitarist
      - bassist
```

- Natural key for a tag is `name`; for a tag group, `name`. (Discourse also assigns numeric ids; the file uses names — see rename caveat.)
- The desired tag set on push = the union of every `tags[].name` and every name listed under any group's `tags:`.

## Discourse API mapping (reference)

- Read: `GET /tags.json`, `GET /tag_groups.json` (admin — returns `tag_names`, `parent_tag_name`, `one_per_topic`, `permissions`).
- Groups: `POST /tag_groups.json`, `PUT /tag_groups/{id}.json` (names in `tag_names` are created implicitly).
- Tag metadata / rename: `PUT /tag/{name}.json`.
- Prune: `DELETE /tag/{name}.json`, `DELETE /tag_groups/{id}.json`.

## Edge cases / open questions

1. **Renames lose data.** A name change in the file is indistinguishable from delete+create, which drops the tag's topic associations. Recommend a dedicated `dsc tag rename <old> <new>` (uses the rename API, preserves associations) rather than expressing renames through pull/push. Group renames have the same issue unless matched by id, which the file does not carry.
2. **paid/unpaid exclusivity.** If these should be mutually exclusive on a topic, they need their own group with `one_per_topic: true`. The starter file keeps them in a non-exclusive `Other` group — confirm the desired behaviour.
3. **Tag-creation permissions (complementary, not part of this command).** Set `min_trust_to_create_tag` via `dsc setting` so ordinary users can apply existing tags but not spawn new junk tags — directly supports the forum's signal-to-noise goal.

## Known bugs (observed 2026-07-01, yorkmusic.org, Discourse 2026.7.0-latest)

> **Status: both fixed (unreleased).** `delete_tag` now uses the singular
> `/tag/{name}.json` endpoint; `tag_push` reconciles tag **groups first** (which
> materialise their tags) and only then sets descriptions, and a desired tag
> that belongs to no group and does not exist is reported (no silent 404 abort).
> Re-verified end-to-end on koloki-demo: create-via-group + description + a
> `--prune` delete that actually removes the tag. See `plan_tags` in
> `src/commands/tag.rs` and its unit tests.

Two defects in the implemented `tag push` / tag delete path, found while applying
`tags.yaml` to a live install. These are bugs (the spec promises behaviour the
implementation does wrong), not gaps.

### 1. `tag push` cannot create a tag that does not yet exist

`tag_push` "creates" a new tag by calling `update_tag(name, desc)`, which does
`PUT /tag/{name}.json` (`src/api/tags.rs:145`). That endpoint only updates an
**existing** tag's description; for a tag that does not yet exist it returns
`404 Not Found`, aborting the whole push on the first new tag:

```
$ dsc tag push -n yorkmusic tags.yaml   # dry-run shows "+ create tag: acoustic"
$ dsc tag push yorkmusic tags.yaml
Error: creating/updating tag 'acoustic'
Caused by: update tag failed with 404 Not Found …
```

Discourse has **no standalone create-tag endpoint** that an admin API key can
hit (probed: `POST /tags.json`, `POST /tag.json`, and `PUT /tag/{name}.json` all
return 404 for a non-existent tag). Tags are created **only implicitly** — by a
tag group (`POST /tag_groups.json` with `tag_names` creates the group *and* its
tags, confirmed: returns tag ids) or by being assigned to a topic.

**Recommended fix:** reorder `tag_push` to reconcile **tag groups first** (whose
`POST /tag_groups.json` materialises the tags), then set descriptions on the
now-existing tags. For desired tags that belong to **no** group, there is no
clean create API — either require every tag to live in a group (the
`tags.yaml` here does), or create them by assigning to a throwaway topic, or
document that groupless tags must pre-exist. The push should at minimum not
abort the entire run on a single 404; collect per-tag errors and report.

### 2. `delete_tag` uses the wrong (plural) endpoint

`delete_tag` calls `DELETE /tags/{name}.json` (plural) (`src/api/tags.rs:177`),
which returns `404`. The correct endpoint is `DELETE /tag/{name}.json`
(singular) — confirmed: `DELETE /tag/{name}.json` returns `200 {"success":"OK"}`
and actually removes the tag, while `DELETE /tags/{name}.json` returns 404 and
leaves the tag in place. This breaks `tag push --prune` (and any `tag remove`).

```
DELETE /tags/zzz_probe_a.json  -> 404   (dsc's current call; tag NOT deleted)
DELETE /tag/zzz_probe_a.json   -> 200   (correct; tag deleted)
```

Fix: change the path in `delete_tag` from `/tags/{name}.json` to
`/tag/{name}.json`.

Both were verified against `yorkmusic.org` (Discourse 2026.7.0-latest),
admin-scope API key, 2026-07-01; probe groups/tags were created and cleaned up
during verification.
