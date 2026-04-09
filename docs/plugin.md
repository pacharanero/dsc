# dsc plugin

List, install, and remove plugins via SSH.

## dsc plugin list

```
dsc plugin list <discourse> [--format text|json|yaml]
```

Lists installed plugins on the specified Discourse.

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
