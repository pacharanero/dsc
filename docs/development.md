# Development

## Building

```bash
# Fast feedback build
cargo build

# Optimized build
cargo build --release

# Run locally
cargo run -- --help
```

## Linting

```bash
cargo fmt
cargo clippy
```

## Testing

Standard test suite:

```bash
cargo test
```

Verbose end-to-end output:

```bash
DSC_TEST_VERBOSE=1 cargo test -- --nocapture
```

End-to-end tests hit a real Discourse. Provide credentials in `testdsc.toml` (or point `TEST_DSC_CONFIG` to a file) using the shape shown below; otherwise e2e tests auto-skip.

```toml
[[discourse]]
name = "myforum"
baseurl = "https://forum.example.com"
apikey = "<admin api key>"
api_username = "system"
changelog_topic_id = 123        # optional unless testing update changelog posting
test_topic_id = 456             # topic used by e2e topic tests
test_category_id = 789          # category used by e2e category tests
test_color_scheme_id = 321      # palette used by e2e palette tests
emoji_path = "./smile.png"     # optional; enables emoji add test
emoji_name = "smile"
test_plugin_url = "https://github.com/discourse/discourse-reactions"
test_plugin_name = "discourse-reactions"
test_theme_url = "https://github.com/discourse/discourse-brand-header"
test_theme_name = "discourse-brand-header"
```

## Shell completions

Completion scripts are generated on demand by the binary itself — they are not committed to the repo. Regenerate them for any supported shell with:

```bash
cargo run -- completions zsh  --dir completions/
cargo run -- completions bash --dir completions/
cargo run -- completions fish --dir completions/
```

The `completions/` directory is gitignored. See [docs/completions.md](completions.md) for user-facing installation instructions.

## Release

Releases are automated via Git tags and GitHub Actions using cargo-dist.

1. Update `CHANGELOG.md` with release notes.
2. Ensure your working tree is clean and tests pass.
3. Run `s/release <version>` (for example, `s/release 0.2.0`).
4. The script commits the version bump, tags `v<version>`, and pushes.
5. The `Release` workflow builds and uploads binaries to GitHub Releases.
6. The `crates-io` job publishes the crate (requires `CARGO_REGISTRY_TOKEN`).

## Project layout

- CLI entrypoint and commands: [src/main.rs](../src/main.rs)
- API client and forum interactions: [src/discourse.rs](../src/discourse.rs)
- Config structures and helpers: [src/config.rs](../src/config.rs)
- Utility helpers (slugify, I/O): [src/utils.rs](../src/utils.rs)
- Example configuration: [dsc.example.toml](../dsc.example.toml)
- Specification notes: [spec/spec.md](../spec/spec.md)
