# dsc tag

List every tag on a Discourse, and add/remove tags on a specific topic.

## dsc tag list

```text
dsc tag list <discourse> [--format text|json|yaml]
```

Lists all tags visible to the authenticated user, with the topic count beside each. Default text output is two columns (tag name, count) and is sortable/cuttable.

## dsc tag apply

```text
dsc tag apply <discourse> <topic-id> <tag>
```

Adds the given tag to the specified topic, preserving any existing tags. No-op (and prints a message) if the topic already has the tag. Supports `--dry-run` (or `-n`) to preview the resulting tag set without sending the change.

## dsc tag remove

```text
dsc tag remove <discourse> <topic-id> <tag>
```

Removes the given tag from the specified topic, leaving the others intact. No-op if the tag isn't present. Supports `--dry-run`.

## Notes

- The topic update endpoint replaces the full tag list, so `apply`/`remove` work by GET-then-PUT internally; if you need to set the tag list atomically to a specific value, do it in one step via the API directly.
- Tagging requires either an admin API key or a key whose user is the topic owner. A 403 response usually means the API user lacks edit permission on the topic.

## Examples

```bash
# Tag every search hit with "triage"
dsc search myforum "status:open category:bugs" --format json \
  | jq -r '.[].id' \
  | xargs -I{} dsc tag apply myforum {} triage

# Preview removal without committing
dsc -n tag remove myforum 1525 archived
```
