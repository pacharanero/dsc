# dsc group

List, inspect, and copy groups.

## dsc group list

```
dsc group list <discourse> [--format text|json|yaml]
```

Lists all groups with their IDs, names, and full names.

## dsc group info

```
dsc group info <discourse> <group-id> [--format json|yaml]
```

Shows details for a specific group.

## dsc group members

```
dsc group members <discourse> <group-id> [--format text|json|yaml]
```

Lists members of the specified group.

## dsc group copy

```
dsc group copy <source-discourse> <group-id> [--target <target-discourse>]
```

Copies the specified group. If `--target` is omitted, copies within the same Discourse.

- The copied group name is slugified and suffixed with `-copy` (e.g., `staff` -> `staff-copy`).
- The copied group full name is set to `Copy of <original full name>`.
- All other fields match the source, except the ID which is assigned by Discourse.

`<group-id>` can be found using `dsc group list`.
