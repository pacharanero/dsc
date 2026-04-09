# dsc theme

List, install, remove, pull, push, and duplicate themes.

## dsc theme list

```
dsc theme list <discourse> [--format text|json|yaml]
```

Lists installed themes on the specified Discourse.

## dsc theme install

```
dsc theme install <discourse> <url>
```

Installs a theme using the SSH command template in `DSC_SSH_THEME_INSTALL_CMD`. The template supports `{url}` and `{name}` placeholders.

## dsc theme remove

```
dsc theme remove <discourse> <name>
```

Removes a theme using the SSH command template in `DSC_SSH_THEME_REMOVE_CMD`. The template supports `{name}` and `{url}` placeholders.

## dsc theme pull

```
dsc theme pull <discourse> <theme-id> [<local-path>]
```

Pulls the specified theme into a local JSON file. `<theme-id>` can be found using `dsc theme list`.

If `<local-path>` is omitted, the file is written to the current directory named from the theme name (slugified). The path to the written file is printed to stdout.

## dsc theme push

```
dsc theme push <discourse> <json-path> [<theme-id>]
```

Pushes a local JSON theme file to a Discourse instance.

- If `<theme-id>` is supplied, updates the existing theme and prints the ID.
- If the JSON file contains an `id` field and no `<theme-id>` argument is given, updates that theme.
- Otherwise creates a new theme and prints the new ID.

## dsc theme duplicate

```
dsc theme duplicate <discourse> <theme-id>
```

Duplicates the specified theme and prints the new theme ID. The copy is named `Copy of <original name>` and is not set as the default theme.
