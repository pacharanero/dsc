# dsc backup

Create, list, download, and restore backups.

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

## dsc backup pull

```text
dsc backup pull <discourse> <backup-filename> [<local-path>]
```

Downloads a backup archive to the local filesystem. `<backup-filename>` is the name shown by `dsc backup list`. If `<local-path>` is omitted, the file is saved in the current directory with the same name.

```bash
dsc backup pull myforum discourse-2026-04-17-230000.tar.gz
dsc backup pull myforum discourse-2026-04-17-230000.tar.gz ./backups/
```

## dsc backup push

```text
dsc backup push <discourse> <backup-path>
```

Restores the specified backup (alias: `dsc backup restore`). `<backup-path>` is the backup filename as shown by `dsc backup list`.

Restoration is destructive and irreversible. Use `--dry-run` (or `-n`) to preview the operation before committing:

```bash
dsc --dry-run backup push myforum discourse-2026-04-17-230000.tar.gz
```
