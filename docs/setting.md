# dsc setting

Get and set site settings on a Discourse install. Requires an admin API key and username.

## dsc setting list

```
dsc setting list <discourse> [--format text|json|yaml]
```

Lists all site settings.

## dsc setting get

```
dsc setting get <discourse> <setting> [--format text|json|yaml]
```

Gets the value of a site setting.

## dsc setting set

```text
dsc setting set <discourse> <setting> <value>
```

Updates a site setting.

Add `--dry-run` (or `-n`) to preview the change without sending it. Combine with `--tags` to verify a bulk update before it fans out:

```bash
dsc --dry-run setting set --tags production title "My Forum"
```
