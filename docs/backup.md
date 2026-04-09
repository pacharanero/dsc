# dsc backup

Create, list, and restore backups.

## dsc backup create

```
dsc backup create <discourse>
```

Triggers a backup on the specified Discourse. The backup is created server-side; it is not downloaded locally.

## dsc backup list

```
dsc backup list <discourse> [--format text|markdown|markdown-table|json|yaml|csv]
```

Lists all backups on the specified Discourse. Supports the same formats as `dsc list`.

## dsc backup restore

```
dsc backup restore <discourse> <backup-path>
```

Restores the specified backup. `<backup-path>` is the backup filename as shown by `dsc backup list`.
