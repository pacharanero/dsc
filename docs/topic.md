# dsc topic

Pull, push, and sync individual topics as local Markdown files.

## dsc topic pull

```
dsc topic pull <discourse> <topic-id> [<local-path>]
```

Pulls the specified topic into a local Markdown file.

If `<local-path>` is omitted, the topic is written to a new file in the current directory (named from the topic title). Directories are created as needed.

## dsc topic push

```
dsc topic push <discourse> <local-path> <topic-id>
```

Pushes the local Markdown file up to the specified topic, updating it with the file contents.

## dsc topic sync

```
dsc topic sync <discourse> <topic-id> <local-path> [--yes]
```

Intelligently syncs the topic with the local Markdown file, using the most recently modified version as the source of truth.

Timestamps of both copies are shown before proceeding. Pass `--yes` (or `-y`) to skip the confirmation prompt.
