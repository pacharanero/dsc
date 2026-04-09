# dsc import

Imports Discourses from a file or stdin.

```
dsc import [<path>]
```

## Supported formats

- **Text file** — one Discourse URL per line.
- **CSV file** — columns: `name, url, tags`.

If `<path>` is omitted, input is read from stdin.

`dsc` will attempt to populate the `name` and `fullname` fields by querying each Discourse URL for the site title.

## Examples

```bash
# Import from a text file of URLs
dsc import urls.txt

# Import from CSV
dsc import forums.csv

# Pipe from stdin
echo "https://forum.example.com" | dsc import
```
