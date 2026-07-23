# dsc plugin

List, install, and remove plugins via SSH.

## dsc plugin list

```
dsc plugin list <discourse> [--format text|json|yaml] [--verbose]
```

Lists installed plugins on the specified Discourse. `-v`/`--verbose` includes additional fields where supported.

## dsc plugin install

```
dsc plugin install <discourse> <url>
```

Installs a plugin using the SSH command template in `DSC_SSH_PLUGIN_INSTALL_CMD`. The template supports `{url}` and `{name}` placeholders.

## dsc plugin remove

```
dsc plugin remove <discourse> <name>
```

Removes a plugin using the SSH command template in `DSC_SSH_PLUGIN_REMOVE_CMD`. The template supports `{name}` and `{url}` placeholders.
