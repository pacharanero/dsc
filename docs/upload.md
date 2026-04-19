# dsc upload

Upload a file (typically an image) to a Discourse install. Returns the short `upload://…` URL that can be embedded in topic and reply Markdown.

```text
dsc upload <discourse> <file> [--upload-type composer] [--format text|json|yaml]
```

In default text mode, prints just the short URL — designed to be captured into a variable or piped:

```bash
url=$(dsc upload myforum ./diagram.png)
echo "Posted ![diagram]($url)" | dsc topic reply myforum 1525
```

`--upload-type` controls Discourse's `type` field. `composer` (the default) is for embedding in posts. Others: `avatar`, `profile_background`, `card_background`, `custom_emoji`.

Use `--format json` for the full upload payload (id, full URL, filesize, dimensions if applicable).

## Examples

```bash
# Get the short URL for a screenshot
dsc upload myforum ./screenshot.png
# upload://a1B2c3D4e5F6.png

# Upload and open the resulting URL in the browser
dsc upload myforum ./diagram.png --format json | jq -r .url | xargs xdg-open

# Inline upload into a reply, all in one shell line
echo "Build output:\n\n![log]($(dsc upload myforum ./build.log))" \
  | dsc topic reply myforum 1525
```
