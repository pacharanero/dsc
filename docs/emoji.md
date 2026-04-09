# dsc emoji

Manage custom emoji on a Discourse install.

Requires an admin API key and username.

## dsc emoji add

```
dsc emoji add <discourse> <emoji-path> [emoji-name]
```

Adds a new emoji from a local image file. If `emoji-name` is omitted, the filename stem is used (slugified; dashes converted to underscores).

If `emoji-path` is a directory, uploads all `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg` files using the filename stem as the emoji name.

If your instance requires a `client_id` query parameter for admin emoji endpoints, set `DSC_EMOJI_CLIENT_ID` to append it automatically.

## dsc emoji list

```
dsc emoji list <discourse> [--format text|json|yaml] [--inline]
```

Lists custom emojis (name + URL).

Flags:

- `--inline` (or `-i`) — render emoji images inline in supported terminals.
  - Override detection with `DSC_EMOJI_INLINE_PROTOCOL=iterm2|kitty|off`.
