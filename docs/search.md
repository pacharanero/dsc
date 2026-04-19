# dsc search

Search topics on a Discourse install.

```text
dsc search <discourse> <query> [--format text|json|yaml]
```

Hits `/search.json?q=…` and prints the matching topics. The query is passed through verbatim, so any Discourse search filter syntax works (`status:open`, `category:foo`, `tags:bug`, `@user`, etc.).

Default text output is one topic per line, ID first — easy to pipe into `awk` or `cut`:

```bash
dsc search myforum "release notes"
# 1525  Daily bookmarks
#  789  Release notes — March 2026

dsc search myforum "release notes" | awk '{print $1}'   # IDs only
dsc search myforum "release notes" --format json        # full structured output
```

Each result includes `id`, `title`, `slug`, `posts_count`, `category_id`, and `tags`.

## Examples

```bash
# Find all open topics tagged "bug"
dsc search myforum "tags:bug status:open"

# Find recent posts by a specific user
dsc search myforum "@alice after:2026-01-01"

# Pull every match into Markdown
dsc search myforum "release notes" --format json \
  | jq -r '.[].id' \
  | xargs -I{} dsc topic pull myforum {}
```
