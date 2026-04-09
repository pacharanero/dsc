# dsc

A Discourse CLI tool written in Rust. Manage multiple Discourse forums from your terminal — track installs, run upgrades over SSH, manage emojis, perform backups, and sync topics and categories as local Markdown.

Most functionality uses the Discourse REST API. `dsc update` runs remote rebuilds via SSH.

## Features

- Track any number of Discourse installs via a single config file.
- Manage categories, topics, settings, and groups across installs.
- Run rebuilds over SSH and optionally post changelog updates.
- Import from text or CSV, or add installs ad-hoc.
- Pull/push individual topics or whole categories as Markdown.
- Upload custom emojis in bulk.
- List, install, and remove themes and plugins.
- Create, list, and restore backups.

## Installation

Prerequisites: a recent Rust toolchain (edition 2024; install via [rustup](https://rustup.rs)).

```bash
# Clone and install
git clone https://github.com/bawmedical/dsc.git
cd dsc
cargo install --path .
```

Or build without installing:

```bash
cargo build --release
./target/release/dsc --help
```

## Quick start

```bash
# Create a config file
cat > dsc.toml <<'EOF'
[[discourse]]
name = "myforum"
baseurl = "https://forum.example.com"
apikey = "<api key>"
api_username = "system"
ssh_host = "forum.example.com"
changelog_topic_id = 123
EOF

# List configured forums
dsc list

# Pull a topic into Markdown for editing
dsc topic pull myforum 42

# Push the edited topic back up
dsc topic push myforum ./topic-title.md 42

# Update a forum over SSH
dsc update myforum
```

## Documentation

- [Configuration](docs/configuration.md) — config file format, search order, field reference
- **Commands:**
  - [list](docs/list.md) — list and filter installs
  - [open](docs/open.md) — open a Discourse in the browser
  - [add](docs/add.md) — add installs to config
  - [import](docs/import.md) — import installs from file or stdin
  - [update](docs/update.md) — run OS and Discourse updates over SSH
  - [emoji](docs/emoji.md) — upload and list custom emoji
  - [topic](docs/topic.md) — pull, push, and sync topics as Markdown
  - [category](docs/category.md) — list, pull, push, and copy categories
  - [palette](docs/palette.md) — list, pull, and push colour palettes
  - [plugin](docs/plugin.md) — list, install, and remove plugins
  - [theme](docs/theme.md) — list, install, remove, pull, push, and duplicate themes
  - [group](docs/group.md) — list, inspect, and copy groups
  - [backup](docs/backup.md) — create, list, and restore backups
  - [setting](docs/setting.md) — get and set site settings
- [Shell completions](docs/completions.md) — bash, zsh, and fish
- [Development](docs/development.md) — building, testing, releasing, project layout

## License

MIT. See [LICENSE](LICENSE).
