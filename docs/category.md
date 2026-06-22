# dsc category

List, pull, push, and copy categories.

## dsc category list

```
dsc category list <discourse> [--format text|json|yaml] [--tree]
```

Lists all categories with their IDs and names.

Flags:

- `--tree` — print categories in a hierarchy, with subcategories indented under parents.

## dsc category pull

```
dsc category pull <discourse> <category-id-or-slug> [<local-path>]
```

Pulls the category into a directory of Markdown files. If `<local-path>` is omitted, writes to a new folder in the current directory (named from the category slug/name). Files are named from topic titles.

Each file gets YAML front matter binding it to its remote topic:

```markdown
---
title: Dependency management
topic_id: 412
url: https://forum.example.com/t/dependency-management/412
pulled_at: 2026-06-22T09:19:00Z
---

[the topic's first post follows here as Markdown]
```

`category push` reads `topic_id` from this block to route updates by ID, so renaming the file or editing the title no longer risks creating a duplicate topic. The front matter is local-only metadata: `dsc` strips it before sending content to Discourse, so the `---` block never reaches the published post. Files without front matter (e.g. ones you author by hand) still work - they fall back to slug/title matching.

## dsc category push

```text
dsc category push [OPTIONS] <discourse> <category-id-or-slug> <local-path>
```

Pushes local Markdown files up to the category, updating existing topics and (by default) creating new ones for files with no remote match. Files are matched by `topic_id` front matter first, falling back to slug/title matching.

The push prints a plan with a sigil per file: `~` (update), `+` (create), `=` (unchanged, skipped). Bodies byte-identical to the remote post (ignoring trailing whitespace) are reported `=` and not re-sent.

Flags:

- `-n`, `--dry-run` — print the plan without writing anything. Run this first and review before pushing for real.
- `--updates-only` — only update existing topics; error with a hint instead of silently creating a new topic when a file has no remote match. Use for curated categories where accidental topic creation must be impossible.
- `--no-bump` — update posts without bumping their topics to the top of the category activity feed (sends `post[no_bump]=true`). Use for silent bulk maintenance edits.
- `--skip-revision` — update posts without recording an edit-history revision (sends `post[skip_revision]=true`). Suppresses the online audit trail; use sparingly.

### Recommended workflow

```text
dsc category pull forum 34 forum-export/      # snapshot (front matter embedded)
# edit files in forum-export/, commit to git for an offline audit trail
dsc category push -n forum 34 forum-export/   # review the plan
dsc category push forum 34 forum-export/      # apply
```

## dsc category copy

```
dsc category copy <source-discourse> <category-id-or-slug> [--target <target-discourse>]
```

Copies the specified category. If `--target` is omitted, copies within the same Discourse.

- The copied category name is set to `Copy of <original category name>`.
- The copied category slug is suffixed with `-copy` (e.g., `staff` -> `staff-copy`).
- All other fields match the source, except the ID which is assigned by Discourse.

`<category-id-or-slug>` can be found using `dsc category list`.
