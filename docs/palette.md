# dsc palette

List, pull, and push colour palettes (color schemes).

## dsc palette list

```
dsc palette list <discourse> [--format text|json|yaml]
```

Lists available colour palettes on the specified Discourse.

## dsc palette pull

```
dsc palette pull <discourse> <palette-id> [<local-path>]
```

Exports the specified palette to a local JSON file. If `<local-path>` is omitted, writes `palette-<id>.json` in the current directory.

## dsc palette push

```
dsc palette push <discourse> <local-path> [<palette-id>]
```

Updates the specified palette with the colors in the local file. If `<palette-id>` is omitted, a new palette is created and the file is updated with the new ID.
