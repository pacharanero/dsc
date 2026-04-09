# dsc add

Adds one or more Discourses to `dsc.toml`, creating one entry per name.

```
dsc add <name>,<name>,... [--interactive]
```

## Modes

- **Default** (non-interactive) appends a full `[[discourse]]` template entry for each name, including all known fields, using placeholders (`""` for strings, `[]` for lists, `0` for numbers). These placeholders are treated as unset when the config is loaded.
- **Interactive** (`--interactive` or `-i`) prompts for base URL, API key, username, tags, ssh_host, and changelog_topic_id. Fields can be left blank to stay unset.

## Examples

```bash
# Add two forums with template entries to fill in later
dsc add forum-a,forum-b

# Add interactively with prompts
dsc add my-forum --interactive
```
