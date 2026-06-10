# `dsc topic pull` - full thread export

Spec for pulling an entire topic thread (all posts, not just the OP) to a local Markdown file. Goal: make `dsc topic pull` useful for reading, archiving, and summarising a thread - not just for the pull/push OP-editing workflow. Driver: real-world use by an LLM agent reading a forum thread to draft a response.

## Motivation

`dsc topic pull <discourse> <topic_id>` currently writes only the first post (the OP) to a local file. This is the right behaviour for the pull → edit → push workflow, but it makes the command useless for any read-oriented use case: reading a long thread, archiving a discussion, feeding a full conversation to an LLM, or producing a human-readable snapshot.

The workaround today is two sequential `curl` calls against the Discourse JSON API with manual pagination, then stripping HTML from the `cooked` field because `include_raw=1` on the topic endpoint returns raw only for the first page's posts:

```
GET /t/364.json?include_raw=1          # posts 1-20 with raw
GET /t/364.json?page=2&include_raw=1   # posts 21+ with raw
```

This workaround is fiddly, requires knowing the pagination boundary, and gives cooked HTML rather than raw Markdown for later-page posts unless `include_raw=1` is set consistently.

## Current state (as of 2026-06-10)

`dsc topic pull` calls `client.fetch_topic(topic_id, true)` which hits `/t/{id}.json?include_raw=1` and returns the `TopicResponse`. It then does:

```rust
topic.post_stream.posts.get(0).and_then(|p| p.raw.clone())
```

Only the OP raw content is written. The `PostStream` model has `posts: Vec<Post>` but does not capture `post_stream.stream` (the flat array of all post IDs that Discourse includes in the response). Posts beyond the first page (Discourse returns 20 per page) are not fetched at all.

`topic push` similarly targets only the OP - that behaviour is correct and should not change.

## Proposed CLI surface

```text
dsc topic pull <discourse> <topic_id> [local_path]   [--full]
```

- **Without `--full`** (current behaviour): write only the OP raw Markdown to `<local_path>`. No change.
- **With `--full`**: fetch all posts in the thread, write a single Markdown file containing all posts in order, each demarcated by a heading with post number, username, and timestamp.

Output format with `--full`:

```markdown
---
title: Sitekit, eRedBook and Harris Health Alliance Acquisition
topic_id: 364
url: https://forum.rcpch.tech/t/sitekit-eredbook-and-harris-health-alliance-acquisition-24-03-2026/364
posts_count: 27
pulled_at: 2026-06-10T11:34:00Z
---

## Post 1 · pacharanero · 2026-03-24

[raw content of post 1]

---

## Post 2 · pacharanero · 2026-03-25

[raw content of post 2]

---
```

No `push` counterpart is needed or in scope for `--full`. A full-thread file is read-only output.

## Reference: API calls observed in the field

Tested against forum.rcpch.tech (Discourse 3.x), topic 364, 27 posts.

```
GET /t/364.json?include_raw=1
Api-Key: <redacted>
Api-Username: pacharanero

→ 200 OK
{
  "title": "Sitekit, eRedBook and Harris Health Alliance Acquisition 24-03-2026",
  "slug": "sitekit-eredbook-and-harris-health-alliance-acquisition-24-03-2026",
  "posts_count": 27,
  "post_stream": {
    "posts": [ /* first 20 posts, each with "raw" field present */ ],
    "stream": [1111, 1112, 1113, ..., 1137]   /* all 27 post IDs */
  }
}
```

```
GET /t/364.json?page=2&include_raw=1
Api-Key: <redacted>
Api-Username: pacharanero

→ 200 OK
{
  "post_stream": {
    "posts": [ /* posts 21-27, each with "raw" field present */ ]
  }
}
```

The `stream` array (all post IDs) is present on page 1 only. Page size is 20. The `?page=N` parameter is 1-indexed and implicit page 1 is the default. Alternatively, specific posts can be fetched by ID via:

```
GET /t/{id}/posts.json?post_ids[]=1111&post_ids[]=1112&include_raw=1
```

This avoids page arithmetic and is preferable when `stream` is available - fetch all IDs from `stream`, chunk into batches of ~20, request each batch. This approach is used by the Discourse JS client internally.

### Model changes needed

`PostStream` needs a new optional field:

```rust
pub struct PostStream {
    pub posts: Vec<Post>,
    #[serde(default)]
    pub stream: Vec<u64>,   // all post IDs; present on first-page response only
}
```

`Post` needs `username` and `created_at` for the output heading:

```rust
pub struct Post {
    pub id: u64,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}
```

(`username` is already returned by the API but not currently captured in the model.)

## Phases

### Phase 1 - blocking

- [ ] Add `stream: Vec<u64>` to `PostStream` model
- [ ] Add `username: Option<String>` to `Post` model (already in API response, just not modelled)
- [ ] Add `fetch_topic_all_posts(topic_id)` to `DiscourseClient`: fetch page 1, extract `stream`, chunk remaining IDs, batch-fetch via `/t/{id}/posts.json?post_ids[]=…&include_raw=1`, merge into ordered `Vec<Post>`
- [ ] Add `--full` flag to `dsc topic pull` CLI
- [ ] Write full-thread Markdown output (YAML frontmatter + `## Post N · username · date` headings + raw body + `---` separators)
- [ ] `topic_pull` without `--full`: no behaviour change

### Phase 2 - iteration ergonomics

- [ ] `--since <post_number>` - pull only posts from post N onwards (useful for following a thread over time)
- [ ] `--format json` - emit structured JSON (array of `{post_number, username, created_at, raw}`) for piping to other tools / LLMs

## Backward compatibility

No change to the default `dsc topic pull` behaviour. `--full` is additive. The model changes (`stream`, `username`) add optional fields with `#[serde(default)]` and cannot break existing deserialisation.

## Out of scope

- `dsc topic push --full`: a full-thread file is a read-only snapshot; replies are handled by `dsc topic reply`.
- Fetching rendered HTML (`cooked`) - raw Markdown is sufficient and more useful for LLM and editing workflows.
- Streaming output for very large threads - page-at-a-time batch fetching is good enough.
- Diffing thread snapshots over time - that is a separate archiving concern.
