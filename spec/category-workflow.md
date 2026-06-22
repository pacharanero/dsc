# `dsc category` pull/push workflow — three gaps

> **Status: Planned.** Three related gaps surfaced from a real-world offline
> playbook sync workflow against `forum.rcpch.tech`. All three affect the
> same `category pull` / `category push` command pair. They are grouped here
> because Gap 2 and Gap 3 are both downstream consequences of Gap 1.

Spec for three missing features in `dsc category pull` and `dsc category push`:

1. **`category pull` does not embed topic IDs** — pulled files have no YAML
   front matter, so the local file has no durable binding to its remote topic.
2. **`category push` ignores `--dry-run`** — the flag is parsed at CLI level
   but never passed into `category_push()`, so the push always executes live.
3. **`category push` silently creates new topics on slug mismatch** — when
   a local file cannot be matched to a remote topic, a new topic is created
   without warning instead of erroring or skipping.

## Context: the real-world driver

`playbook.rcpch.tech` is being migrated to use `forum.rcpch.tech/c/playbook`
(category 34) as its canonical home. The workflow is:

1. `dsc category pull rcpch 34 forum-export/` — snapshot all 27 topics to a
   Git-tracked local directory.
2. Edit files in `forum-export/` and commit changes to Git.
3. `dsc category push rcpch 34 forum-export/` — push edits back to Discourse.

The git history provides an offline audit trail. Discourse's built-in edit
revisions provide an online one. The two together give full provenance.

This workflow has a hard governance constraint: **no push to the forum without
human review of exactly what will change, and no accidental topic creation or
deletion.** Both of those requirements are blocked by the three gaps below.

Tested against `forum.rcpch.tech` (Discourse stable, June 2026), category 34,
27 topics.

---

## Gap 1 — `category pull` does not embed topic IDs in output files

### What happens now

`category_pull` (in `src/commands/category.rs`) iterates over every topic in
the category, fetches the first post's raw content, and writes it to a file
named by `slugify(&topic.title)`:

```rust
// src/commands/category.rs  category_pull()
for topic in category.topic_list.topics {
    let topic_detail = client.fetch_topic(topic.id, true)?;
    let raw = topic_detail.post_stream.posts.get(0)
        .and_then(|p| p.raw.clone())
        .unwrap_or_default();
    let filename = format!("{}.md", slugify(&topic.title));
    write_markdown(&dir.join(filename), &raw)?;
}
```

At this point the code has `topic.id` in scope, but writes only the raw
Markdown body. No YAML front matter is prepended. The mapping from local file
→ Discourse topic ID is lost the moment the file is written.

### Why this matters

`category_push` matches local files to remote topics using
`find_topic_match()` (same file):

```rust
fn find_topic_match<'a>(
    topics: &'a [TopicSummary],
    title: &str,
    path: &Path,
) -> Option<&'a TopicSummary> {
    let slug = slugify(title);
    topics.iter().find(|topic| {
        topic.slug == slug
            || topic.title.eq_ignore_ascii_case(title)
            || path.file_stem()
                .map(|s| s.to_string_lossy().eq_ignore_ascii_case(&topic.slug))
                .unwrap_or(false)
    })
}
```

This relies entirely on the title or filename continuing to match the remote
topic's slug. If **any** of the following happens, the match fails silently
and a **new duplicate topic is created**:

- The topic's title is edited locally (slug changes)
- `slugify()` produces a different result than Discourse's own slugifier for
  edge cases (accented characters, punctuation, very long titles)
- A file is renamed for organisational reasons
- The topic's slug is changed directly in Discourse

Gap 3 describes the silent-create consequence; the root fix is here in Gap 1.

### Precedent

`dsc topic pull --full` (implemented in v0.10.11, `src/commands/topic.rs`
`render_full_thread()`) already writes YAML front matter with `topic_id` and
`url`. The same pattern should be applied to `category pull`.

### What is needed

Prepend YAML front matter to every file written by `category pull`, using
the topic ID, URL, and title that are already in scope:

```yaml
---
title: Dependency management
topic_id: 412
url: https://forum.rcpch.tech/t/dependency-management/412
pulled_at: 2026-06-22T09:19:00Z
---

[raw markdown body follows, unchanged]
```

`category push` must then:

1. Detect YAML front matter in the local file (fenced by `---\n … \n---\n`).
2. Parse `topic_id` from it.
3. Use `topic_id` directly to route the update (`client.fetch_topic(topic_id,
   true)` → `client.update_post(post.id, &stripped_body)`).
4. Strip the front matter from the body before sending to Discourse (so the
   `---` block does not appear in the published post).
5. Fall back to the existing slug/title matching only when front matter is
   absent (backwards compatibility with pre-front-matter files).

`read_markdown` (`src/utils.rs`) currently returns the raw file content
unchanged. A companion `strip_frontmatter(raw: &str) -> (Option<FrontMatter>,
String)` helper should be added to `utils.rs` and used by `category push`
(and by `topic push`, which has the same blind-read problem if someone
manually adds front matter to a file).

### Reference: API calls observed in the field

Category topic list (already used by `category_pull`):

```
GET /c/playbook/34.json
Api-Key: <redacted>
Api-Username: pacharanero

→ 200 OK
{
  "topic_list": {
    "topics": [
      { "id": 394, "title": "About this Playbook", "slug": "about-this-playbook" },
      { "id": 412, "title": "Dependency management", "slug": "dependency-management" },
      ...
    ]
  }
}
```

The `id` field is present on every `TopicSummary` in the list response and is
already modelled in `TopicSummary` (`src/api/models.rs`). No new API calls are
needed — the fix is purely about propagating data that is already fetched.

---

## Gap 2 — `category push` ignores `--dry-run`

### What happens now

The global `--dry-run` / `-n` flag is parsed at the top level in
`src/main.rs`:

```rust
// src/main.rs (line ~43)
let dry_run = cli.dry_run;
```

But the call site for `category_push` omits the argument:

```rust
// src/main.rs (line ~226)
} => commands::category::category_push(&config, &discourse, &category, &local_path),
//                                                                                 ^
//                                                   dry_run is NOT passed here
```

Compare with `topic push`, which correctly passes `dry_run`:

```rust
// src/main.rs (line ~144)
} => commands::topic::topic_push(&config, &discourse, topic_id, &local_path, dry_run),
```

And `category_push`'s function signature has no `dry_run` parameter at all:

```rust
// src/commands/category.rs (line ~144)
pub fn category_push(
    config: &Config,
    discourse_name: &str,
    category: &str,
    local_path: &Path,
    // dry_run: bool  ← missing
) -> Result<()>
```

### Why this matters

The governance constraint for this workflow is: **always dry-run first,
review the plan, then execute.** Without a working `--dry-run`, there is no
safe preview step. Running `dsc category push --dry-run` today silently
executes a live push. Users who read `dsc --help` and see `--dry-run` listed
as a global flag have no indication it is not honoured by this subcommand.

### What is needed

1. Add `dry_run: bool` to `category_push()`'s signature.
2. Pass `dry_run` from the call site in `main.rs`.
3. In the push loop, gate all mutating operations on `!dry_run`:

```rust
if dry_run {
    if let Some(topic) = find_topic_match(&topics, &title, &path) {
        let url = format!("{}/t/{}/{}", base_url, topic.slug, topic.id);
        println!("[dry-run] ~ would update topic {} \"{}\" ({}) with {} bytes",
            topic.id, title, url, raw.len());
    } else {
        println!("[dry-run] + would create new topic \"{}\" ({} bytes) in category {}",
            title, raw.len(), category_id);
    }
} else {
    // existing update / create logic
}
```

The `~` (change) / `+` (create) / `=` (unchanged) sigils are already used
elsewhere in `dsc` dry-run output (e.g. `setting push --dry-run`) — use the
same convention for consistency.

Optionally, for `=` (unchanged): compare the local body against the fetched
remote body and emit `=` if they are byte-identical. This avoids
no-op API writes and makes the dry-run output meaningful even when nothing
has changed.

---

## Gap 3 — `category push` silently creates new topics on slug mismatch

### What happens now

When `find_topic_match()` returns `None`, `category_push` immediately creates
a new topic:

```rust
// src/commands/category.rs  category_push()
if let Some(topic) = find_topic_match(&topics, &title, &path) {
    // update existing topic
    client.update_post(post.id, &raw)?;
} else {
    // ← no warning, no --no-create guard, just creates
    let topic_id = client.create_topic(category_id, &title, &raw)?;
    topics.push(TopicSummary { id: topic_id, title: title.clone(), slug: slugify(&title) });
}
```

There is no flag to suppress this behaviour, no warning to the operator, and
no dry-run output (see Gap 2). The only signal of creation is the absence of
an error.

### Why this matters

Accidental topic creation is hard to undo cleanly. Discourse does not expose
a delete-topic API to regular API clients; topics must be archived/unlisted
rather than deleted. In a curated category like a Playbook, orphaned duplicate
topics pollute the index and confuse readers. The governance rule for this
workflow is that **no new topics should ever be created without deliberate
human intent**.

This gap is primarily addressed by Gap 1 (with `topic_id` in front matter,
the slug match is no longer needed for known topics). But an explicit guard
is still valuable for cases where a completely new file is introduced.

### What is needed

Add an `--updates-only` flag to `dsc category push`:

```text
dsc category push [OPTIONS] <DISCOURSE> <CATEGORY> <LOCAL_PATH>

Options:
  --updates-only   Only update existing topics; error if a local file has no
                   remote match instead of creating a new topic
  -n, --dry-run    ...
```

When `--updates-only` is set and `find_topic_match()` returns `None`, emit a
clear error (not just a warning) and stop:

```
error: no matching topic found for "my-new-file.md" (title: "My New File")
hint: remove --updates-only to allow new topic creation, or check the filename matches an existing topic slug
```

The default behaviour (create on mismatch) is preserved so existing
workflows are not broken.

---

## Implementation order

Implement in this order — each phase unblocks the next:

### Phase 1 — `category pull` embeds front matter (Gap 1, pull side)

- [ ] Add `strip_frontmatter(raw: &str) -> (Option<HashMap<String, String>>, String)` helper to `src/utils.rs`. Parse the `---`-fenced block at the start of the file into key→value pairs; return the stripped body as the second element.
- [ ] Update `category_pull()` to prepend YAML front matter (`title`, `topic_id`, `url`, `pulled_at`) before writing each file.
- [ ] Add a `current_utc_iso8601()` call (already exists in `src/commands/topic.rs` — consider moving to `utils.rs` and sharing it).

### Phase 2 — `category push` reads front matter and routes by ID (Gap 1, push side)

- [ ] In `category_push()`, after reading each file, call `strip_frontmatter()` to separate the body from the front matter.
- [ ] If `topic_id` is present in front matter, use it directly to route the update (skip `find_topic_match()`).
- [ ] Pass only the stripped body (not the front matter) to `client.update_post()` and `client.create_topic()`.
- [ ] Keep `find_topic_match()` as a fallback for files without front matter.

### Phase 3 — working `--dry-run` for `category push` (Gap 2)

- [ ] Add `dry_run: bool` parameter to `category_push()`.
- [ ] Pass `dry_run` at the call site in `src/main.rs`.
- [ ] Implement dry-run output using `~` / `+` / `=` sigils.
- [ ] Optionally: skip the `client.update_post()` call when body is
  byte-identical to remote (avoids no-op edits and `=` output is honest).

### Phase 4 — `--updates-only` guard (Gap 3)

- [ ] Add `updates_only: bool` parameter to `category_push()` and wire it from CLI.
- [ ] When `updates_only` is true and no match is found, emit a structured error instead of calling `create_topic()`.

---

## Backward compatibility

- Files written by the current `category pull` (no front matter) continue to
  work — `strip_frontmatter()` returns an empty map and the full file content
  as the body, and `find_topic_match()` is used as before.
- Default `category push` behaviour (create on mismatch) is unchanged unless
  `--updates-only` is explicitly set.
- `topic push` is not changed (it already takes an explicit topic ID).
- `topic pull` (non-`--full`) is not changed — it is already used for the
  OP-edit workflow and adding front matter there is a separate decision.

## Out of scope

- Adding front matter to `topic pull` (non-`--full`): related but separate;
  worth a follow-up if the same ID-binding problem surfaces for single-topic
  workflows.
- Title editing via `category push`: the front matter `title` field is
  metadata only; changing the title of a Discourse topic requires a separate
  API call (`PUT /t/{slug}/{id}.json` with `title`) and is not in scope here.
- Topic deletion: explicitly out of scope and should remain so. `dsc` should
  never delete topics in a category push workflow.
