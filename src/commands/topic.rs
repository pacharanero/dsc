use crate::api::DiscourseClient;
use crate::api::PostEditOptions;
use crate::api::TopicResponse;
use crate::cli::ListFormat;
use crate::commands::common::{emit_result, ensure_api_credentials, select_discourse};
use crate::config::Config;
use serde_json::json;
use crate::utils::{
    current_utc_iso8601, read_markdown, resolve_topic_path, strip_frontmatter, write_markdown,
    yaml_scalar,
};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

pub fn topic_pull(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    local_path: Option<&Path>,
    full: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    if full {
        let topic = client.fetch_topic_all_posts(topic_id)?;
        let title = topic_display_title(&topic, topic_id);
        let body = render_full_thread(&topic, topic_id, &discourse.baseurl);
        let target = resolve_topic_path(local_path, &title, &std::env::current_dir()?)?;
        write_markdown(&target, &body)?;
        println!(
            "Topic pulled (full thread, {} posts) to: {}",
            topic.post_stream.posts.len(),
            target.display()
        );
        return Ok(());
    }

    let topic = client.fetch_topic(topic_id, true)?;
    let raw = topic
        .post_stream
        .posts
        .first()
        .and_then(|p| p.raw.clone())
        .ok_or_else(|| anyhow!("topic has no raw content"))?;
    let title = topic_display_title(&topic, topic_id);
    let target = resolve_topic_path(local_path, &title, &std::env::current_dir()?)?;
    write_markdown(&target, &raw)?;
    println!("Topic pulled to: {}", target.display());
    Ok(())
}

/// Pick a stable display string for the topic - title, then slug, then a
/// `topic-N` fallback. Used for both filename derivation and Markdown
/// frontmatter.
fn topic_display_title(topic: &TopicResponse, topic_id: u64) -> String {
    topic
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.to_string())
        .or_else(|| {
            topic
                .slug
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| format!("topic-{}", topic_id))
}

/// Render every post in `topic` as a single Markdown document with YAML
/// frontmatter (title / topic_id / url / posts_count / pulled_at) and
/// per-post `## Post N · username · date` headings separated by `---`
/// horizontal rules.
fn render_full_thread(topic: &TopicResponse, topic_id: u64, baseurl: &str) -> String {
    let title = topic_display_title(topic, topic_id);
    let slug = topic
        .slug
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("topic");
    let base_trimmed = baseurl.trim_end_matches('/');
    let url = format!("{}/t/{}/{}", base_trimmed, slug, topic_id);
    let posts_count = topic.post_stream.posts.len();
    let pulled_at = current_utc_iso8601();

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: {}\n", yaml_scalar(&title)));
    out.push_str(&format!("topic_id: {}\n", topic_id));
    out.push_str(&format!("url: {}\n", url));
    out.push_str(&format!("posts_count: {}\n", posts_count));
    out.push_str(&format!("pulled_at: {}\n", pulled_at));
    out.push_str("---\n\n");

    for (idx, post) in topic.post_stream.posts.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n---\n\n");
        }
        let post_number = post.post_number.unwrap_or((idx + 1) as u64);
        let username = post.username.as_deref().unwrap_or("(unknown)");
        let date = post
            .created_at
            .as_deref()
            .map(format_date_only)
            .unwrap_or_else(|| "(no date)".to_string());
        out.push_str(&format!(
            "## Post {} · {} · {}\n\n",
            post_number, username, date
        ));
        if let Some(raw) = post.raw.as_deref() {
            out.push_str(raw.trim_end());
            out.push('\n');
        } else {
            out.push_str("_(raw content unavailable)_\n");
        }
    }
    out
}

/// Trim an ISO-8601 timestamp like `2026-03-24T11:07:00.123Z` down to the
/// date portion. Leaves anything that doesn't parse cleanly as-is.
fn format_date_only(ts: &str) -> String {
    match ts.find('T') {
        Some(idx) => ts[..idx].to_string(),
        None => ts.to_string(),
    }
}

pub fn topic_push(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    local_path: &Path,
    dry_run: bool,
    edit_opts: PostEditOptions,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let topic = client.fetch_topic(topic_id, true)?;
    let post = topic
        .post_stream
        .posts
        .first()
        .ok_or_else(|| anyhow!("topic has no posts"))?;
    let raw = read_markdown(local_path)?;
    // Strip any YAML front matter so a manually-annotated file (or one carried
    // over from a `category pull`) pushes a clean body — the `---` block is
    // local-only metadata and must never reach the published post.
    let (_front, body) = strip_frontmatter(&raw);
    if dry_run {
        println!(
            "[dry-run] {}: would replace OP of topic {} (post id {}) with {} bytes from {}",
            discourse.name,
            topic_id,
            post.id,
            body.len(),
            local_path.display()
        );
        return Ok(());
    }
    client.update_post(post.id, &body, edit_opts)?;
    Ok(())
}

pub fn topic_sync(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    local_path: &Path,
    assume_yes: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let topic = client.fetch_topic(topic_id, true)?;
    let post = topic
        .post_stream
        .posts
        .get(0)
        .ok_or_else(|| anyhow!("topic has no posts"))?;
    let local_meta =
        fs::metadata(local_path).with_context(|| format!("reading {}", local_path.display()))?;
    let local_mtime = local_meta.modified()?;

    let remote_ts = post
        .updated_at
        .as_deref()
        .or(post.created_at.as_deref())
        .ok_or_else(|| anyhow!("missing remote timestamps"))?;
    let remote_time = chrono::DateTime::parse_from_rfc3339(remote_ts)
        .context("parsing remote timestamp")?
        .with_timezone(&chrono::Utc);

    println!(
        "Local file:  {}",
        chrono::DateTime::<chrono::Utc>::from(local_mtime)
    );
    println!("Remote post: {}", remote_time);

    let pull = remote_time > chrono::DateTime::<chrono::Utc>::from(local_mtime);
    if !assume_yes && !confirm_sync(pull)? {
        return Ok(());
    }

    if pull {
        let raw = post
            .raw
            .clone()
            .ok_or_else(|| anyhow!("missing raw content"))?;
        write_markdown(local_path, &raw)?;
    } else {
        let raw = read_markdown(local_path)?;
        client.update_post(post.id, &raw, PostEditOptions::default())?;
    }

    Ok(())
}

pub fn topic_reply(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    local_path: Option<&Path>,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let raw = read_reply_input(local_path)?;
    if raw.trim().is_empty() {
        return Err(anyhow!("reply body is empty"));
    }

    let post_id = client.create_post(topic_id, &raw)?;
    emit_result(
        format,
        &json!({ "topic_id": topic_id, "post_id": post_id }),
        &format!("Replied to topic {} (post id {})", topic_id, post_id),
    )
}

pub fn topic_new(
    config: &Config,
    discourse_name: &str,
    category_id: u64,
    title: &str,
    local_path: Option<&Path>,
    dry_run: bool,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    if title.trim().is_empty() {
        return Err(anyhow!("topic title is empty"));
    }
    let raw = read_reply_input(local_path)?;
    if raw.trim().is_empty() {
        return Err(anyhow!("topic body is empty"));
    }

    if dry_run {
        return emit_result(
            format,
            &json!({ "dry_run": true, "category_id": category_id, "title": title }),
            &format!(
                "[dry-run] {}: would create topic in category {} titled \"{}\" ({} bytes of body)",
                discourse.name,
                category_id,
                title,
                raw.len()
            ),
        );
    }

    let topic_id = client.create_topic(category_id, title, &raw)?;
    emit_result(
        format,
        &json!({ "topic_id": topic_id, "category_id": category_id }),
        &format!("Created topic {} in category {}", topic_id, category_id),
    )
}

fn read_reply_input(local_path: Option<&Path>) -> Result<String> {
    let from_stdin = match local_path {
        None => true,
        Some(p) => p.as_os_str() == "-",
    };
    if from_stdin {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("reading reply from stdin")?;
        Ok(buf)
    } else {
        let path = local_path.unwrap();
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_date_only, read_reply_input, render_full_thread, topic_display_title};
    use crate::utils::yaml_scalar;
    use crate::api::{Post, PostStream, TopicResponse};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_topic(title: Option<&str>, posts: Vec<Post>, stream: Vec<u64>) -> TopicResponse {
        TopicResponse {
            title: title.map(|s| s.to_string()),
            slug: Some("hello-world".to_string()),
            post_stream: PostStream { posts, stream },
        }
    }

    fn make_post(
        id: u64,
        post_number: Option<u64>,
        username: Option<&str>,
        raw: Option<&str>,
        created_at: Option<&str>,
    ) -> Post {
        Post {
            id,
            post_number,
            username: username.map(|s| s.to_string()),
            raw: raw.map(|s| s.to_string()),
            updated_at: None,
            created_at: created_at.map(|s| s.to_string()),
        }
    }

    #[test]
    fn read_reply_input_reads_from_file() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "hello from file").unwrap();
        let got = read_reply_input(Some(f.path())).unwrap();
        assert_eq!(got.trim(), "hello from file");
    }

    #[test]
    fn read_reply_input_missing_file_surfaces_path_in_error() {
        let bogus = std::path::Path::new("/definitely/does/not/exist.md");
        let err = read_reply_input(Some(bogus)).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("/definitely/does/not/exist.md"));
    }

    #[test]
    fn display_title_prefers_title_then_slug_then_fallback() {
        let t1 = make_topic(Some("My Title"), vec![], vec![]);
        assert_eq!(topic_display_title(&t1, 42), "My Title");

        let t2 = TopicResponse {
            title: Some("  ".to_string()),
            slug: Some("my-slug".to_string()),
            post_stream: PostStream::default(),
        };
        assert_eq!(topic_display_title(&t2, 42), "my-slug");

        let t3 = TopicResponse {
            title: None,
            slug: None,
            post_stream: PostStream::default(),
        };
        assert_eq!(topic_display_title(&t3, 42), "topic-42");
    }

    #[test]
    fn format_date_only_trims_at_t() {
        assert_eq!(format_date_only("2026-03-24T11:07:00Z"), "2026-03-24");
        assert_eq!(format_date_only("2026-03-24"), "2026-03-24");
        assert_eq!(format_date_only(""), "");
    }

    #[test]
    fn yaml_scalar_quotes_when_ambiguous() {
        assert_eq!(yaml_scalar("simple title"), "simple title");
        // Colon triggers quoting.
        assert_eq!(yaml_scalar("a: b"), "\"a: b\"");
        // Leading hash would otherwise read as a comment.
        assert_eq!(yaml_scalar("#hash"), "\"#hash\"");
        // Leading quote forces quoting + escapes inner quotes.
        assert_eq!(yaml_scalar("\"q"), "\"\\\"q\"");
        // Embedded quotes mid-string are fine in plain YAML scalars.
        assert_eq!(yaml_scalar("she said hi"), "she said hi");
    }

    #[test]
    fn render_full_thread_emits_frontmatter_and_per_post_headings() {
        let posts = vec![
            make_post(101, Some(1), Some("alice"), Some("hello"), Some("2026-03-24T11:00:00Z")),
            make_post(102, Some(2), Some("bob"), Some("hi back"), Some("2026-03-25T09:00:00Z")),
        ];
        let topic = make_topic(Some("Hello World"), posts, vec![101, 102]);
        let out = render_full_thread(&topic, 42, "https://forum.example.com/");

        assert!(out.starts_with("---\n"));
        assert!(out.contains("title: Hello World\n"));
        assert!(out.contains("topic_id: 42\n"));
        assert!(out.contains("url: https://forum.example.com/t/hello-world/42\n"));
        assert!(out.contains("posts_count: 2\n"));
        assert!(out.contains("## Post 1 · alice · 2026-03-24\n"));
        assert!(out.contains("## Post 2 · bob · 2026-03-25\n"));
        assert!(out.contains("hello"));
        assert!(out.contains("hi back"));
        assert!(out.contains("\n---\n"), "horizontal rule between posts");
    }

    #[test]
    fn render_full_thread_handles_missing_raw_and_user() {
        let posts = vec![make_post(7, Some(1), None, None, None)];
        let topic = make_topic(None, posts, vec![7]);
        let out = render_full_thread(&topic, 7, "https://x.test");
        assert!(out.contains("(unknown)"));
        assert!(out.contains("(no date)"));
        assert!(out.contains("_(raw content unavailable)_"));
    }

    #[test]
    fn render_full_thread_falls_back_to_index_when_post_number_missing() {
        let posts = vec![make_post(7, None, Some("alice"), Some("body"), None)];
        let topic = make_topic(Some("t"), posts, vec![7]);
        let out = render_full_thread(&topic, 7, "https://x.test");
        // Single post with no post_number → numbered 1 from index.
        assert!(out.contains("## Post 1 · alice"));
    }

}

fn confirm_sync(pull: bool) -> Result<bool> {
    let action = if pull {
        "pull from Discourse"
    } else {
        "push to Discourse"
    };
    print!("Proceed to {}? [y/N]: ", action);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}
