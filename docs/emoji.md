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

Bulk uploads retry automatically on HTTP 429 responses, reading the wait time from the `Retry-After` header, the `extras.wait_seconds` JSON field, or the message body.

If you consistently hit rate limits on large batches, raise `DISCOURSE_MAX_ADMIN_API_REQS_PER_MINUTE` (default 60). This is a Discourse global setting, not a site setting — it is not visible in the Admin UI. On a standard Docker install, set it under `env:` in `/var/discourse/containers/app.yml` and rebuild the container. Nginx-level 429s (HTML body, `nginx` in the response) come from the reverse proxy, not Discourse itself, and must be raised in the proxy config.

## dsc emoji list

```
dsc emoji list <discourse> [--format text|json|yaml] [--inline]
```

Lists custom emojis (name + URL).

Flags:

- `--inline` (or `-i`) — render emoji images inline in supported terminals.
  - Override detection with `DSC_EMOJI_INLINE_PROTOCOL=iterm2|kitty|off`.
