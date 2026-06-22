# dsc topic

Pull, push, and sync individual topics as local Markdown files. Also tag/untag topics.

## dsc topic pull

```
dsc topic pull <discourse> <topic-id> [<local-path>] [--full|-F]
```

Pulls the specified topic into a local Markdown file.

By default writes only the topic's first post (the OP), suitable for the `pull → edit → push` round-trip with `dsc topic push`.

If `<local-path>` is omitted, the topic is written to a new file in the current directory (named from the topic title). Directories are created as needed.

### `--full` (read-only thread snapshot)

```bash
dsc topic pull myforum 364 --full
dsc topic pull myforum 364 thread.md --full
```

Pulls every post in the thread (paginating internally as needed) into a single Markdown file with YAML frontmatter and per-post headings:

```markdown
---
title: Sitekit, eRedBook and Harris Health Alliance Acquisition
topic_id: 364
url: https://forum.example.com/t/sitekit-.../364
posts_count: 27
pulled_at: 2026-06-10T11:34:00Z
---

## Post 1 · alice · 2026-03-24

[raw markdown of post 1]

---

## Post 2 · bob · 2026-03-25

[raw markdown of post 2]
```

A full-thread file is a read-only snapshot. `dsc topic push` still operates on the OP only and does not consume the full-thread format.

Useful for archiving long discussions, feeding a complete conversation to an LLM, or producing a human-readable export.

## dsc topic push

```text
dsc topic push [OPTIONS] <discourse> <topic-id> <local-path>
```

Pushes the local Markdown file up to the specified topic, updating its first post with the file contents. Any leading YAML front matter is stripped before sending, so a file annotated with a `---` block (or one carried over from `category pull`) pushes a clean body.

Flags:

- `-n`, `--dry-run` — describe the edit without sending it.
- `--no-bump` — update the post without bumping the topic to the top of the activity feed (sends `post[no_bump]=true`). Use for silent maintenance edits.
- `--skip-revision` — update without recording an edit-history revision (sends `post[skip_revision]=true`). Suppresses the online audit trail; use sparingly.

## dsc topic sync

```
dsc topic sync <discourse> <topic-id> <local-path> [--yes]
```

Intelligently syncs the topic with the local Markdown file, using the most recently modified version as the source of truth.

Timestamps of both copies are shown before proceeding. Pass `--yes` (or `-y`) to skip the confirmation prompt.

## dsc topic reply

```text
dsc topic reply <discourse> <topic-id> [<local-path>] [--format text|json|yaml]
```

Posts a new reply at the end of the topic. Reads from `<local-path>` if given, otherwise from stdin (equivalent to passing `-`). `--format json` emits `{"topic_id": ..., "post_id": ...}` for scripting.

Examples:

```bash
dsc topic reply myforum 1525 ./note.md
git log --since=yesterday --oneline | dsc topic reply myforum 1525
```

## dsc topic new

```text
dsc topic new <discourse> <category-id> --title <title> [<local-path>] [--format text|json|yaml]
```

Creates a new topic in the given category with the specified title. Reads the body from `<local-path>` if given, otherwise from stdin. `--format json` emits `{"topic_id": ..., "category_id": ...}` for scripting.

Examples:

```bash
dsc topic new myforum 42 --title "Release notes" ./notes.md
df -h | dsc topic new myforum 42 -t "Disk report $(date -I)"
```

## dsc topic tag

```text
dsc topic tag <discourse> <topic-id> <tag>
```

Adds the given tag to the specified topic, preserving any existing tags. No-op (and prints a message) if the topic already has the tag. Supports `--dry-run` (`-n`).

## dsc topic untag

```text
dsc topic untag <discourse> <topic-id> <tag>
```

Removes the given tag from the specified topic, leaving the others intact. No-op if the tag isn't present. Supports `--dry-run`.

```bash
# Tag every search hit with "triage"
dsc search myforum "status:open category:bugs" --format json \
  | jq -r '.[].id' \
  | xargs -I{} dsc topic tag myforum {} triage

# Preview removal without committing
dsc -n topic untag myforum 1525 archived
```
