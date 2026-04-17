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

## dsc category push

```text
dsc category push <discourse> <category-id-or-slug> <local-path>
```

Pushes local Markdown files up to the category, creating or updating topics as necessary.

## dsc category copy

```
dsc category copy <source-discourse> <category-id-or-slug> [--target <target-discourse>]
```

Copies the specified category. If `--target` is omitted, copies within the same Discourse.

- The copied category name is set to `Copy of <original category name>`.
- The copied category slug is suffixed with `-copy` (e.g., `staff` -> `staff-copy`).
- All other fields match the source, except the ID which is assigned by Discourse.

`<category-id-or-slug>` can be found using `dsc category list`.
