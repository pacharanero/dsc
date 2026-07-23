# dsc theme palette

List, pull, and push colour palettes (color schemes).

> **Renamed.** These commands now live under `dsc theme palette`. The old
> top-level `dsc palette …` form still works as a deprecated alias (it prints
> a one-line notice) and will be removed in a future release. Substitute
> `dsc theme palette` for `dsc palette` in the examples below.

## dsc theme palette list

```
dsc theme palette list <discourse> [--format text|json|yaml] [--verbose]
```

Lists available colour palettes on the specified Discourse. `-v`/`--verbose` includes additional fields where supported.

## dsc theme palette pull

```
dsc theme palette pull <discourse> <palette-id> [<local-path>]
```

Exports the specified palette to a local JSON file. If `<local-path>` is omitted, writes `palette-<id>.json` in the current directory.

## dsc theme palette push

```
dsc theme palette push <discourse> <local-path> [<palette-id>]
```

Updates the specified palette with the colors in the local file. If `<palette-id>` is omitted, a new palette is created and the file is updated with the new ID.
