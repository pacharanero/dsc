use crate::api::{CategoryInfo, DiscourseClient, PostEditOptions, TopicSummary};
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, not_found, select_discourse};
use crate::config::Config;
use crate::utils::{
    current_utc_iso8601, ensure_dir, normalize_baseurl, read_markdown, slugify, strip_frontmatter,
    write_markdown, yaml_scalar,
};
use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};

pub fn category_list(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
    verbose: bool,
    tree: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let categories = client.fetch_categories()?;
    let mut flat = Vec::new();
    for category in categories {
        flatten_categories(&category, &mut flat);
    }
    match format {
        ListFormat::Text => {
            if tree {
                if flat.is_empty() && !verbose {
                    println!("No categories found.");
                    return Ok(());
                }
                print_category_tree(&flat);
            } else {
                let unique = unique_categories(flat);
                if unique.is_empty() && !verbose {
                    println!("No categories found.");
                    return Ok(());
                }
                for category in unique {
                    let id = category.id.unwrap_or_default();
                    println!("{} - {}", id, category.name);
                }
            }
        }
        ListFormat::Json => {
            if tree {
                return Err(anyhow!("--tree is only supported with --format text"));
            }
            let unique = unique_categories(flat);
            let raw = serde_json::to_string_pretty(&unique)?;
            println!("{}", raw);
        }
        ListFormat::Yaml => {
            if tree {
                return Err(anyhow!("--tree is only supported with --format text"));
            }
            let unique = unique_categories(flat);
            let raw = serde_yaml::to_string(&unique)?;
            println!("{}", raw);
        }
    }
    Ok(())
}

pub fn category_copy(
    config: &Config,
    source: &str,
    target: Option<&str>,
    category: &str,
    dry_run: bool,
) -> Result<()> {
    let source_discourse = select_discourse(config, Some(source))?;
    let target_name = target.unwrap_or(source);
    let target_discourse = select_discourse(config, Some(target_name))?;
    ensure_api_credentials(source_discourse)?;
    ensure_api_credentials(target_discourse)?;
    let source_client = DiscourseClient::new(source_discourse)?;
    let category_id = resolve_category_id(&source_client, category)?;
    let categories = source_client.fetch_categories()?;
    let category = categories
        .into_iter()
        .find(|cat| cat.id == Some(category_id))
        .ok_or_else(|| not_found("category", category_id))?;
    let mut copied = category.clone();
    copied.name = format!("Copy of {}", category.name);
    copied.slug = format!("{}-copy", category.slug);
    copied.id = None;
    if dry_run {
        println!(
            "[dry-run] would create category \"{}\" (slug: {}) on {}",
            copied.name, copied.slug, target_discourse.name
        );
        return Ok(());
    }
    let target_client = DiscourseClient::new(target_discourse)?;
    let new_id = target_client.create_category(&copied)?;
    let url = format!(
        "{}/c/{}",
        normalize_baseurl(&target_discourse.baseurl),
        new_id
    );
    println!("{}", url);
    Ok(())
}

pub fn category_pull(
    config: &Config,
    discourse_name: &str,
    category: &str,
    local_path: Option<&Path>,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let category_id = resolve_category_id(&client, category)?;
    let category = client.fetch_category(category_id)?;
    let dir = match local_path {
        Some(path) => path.to_path_buf(),
        None => {
            let name = category
                .category
                .as_ref()
                .map(|c| c.slug.clone())
                .unwrap_or_else(|| format!("category-{}", category_id));
            std::env::current_dir()?.join(name)
        }
    };
    ensure_dir(&dir)?;
    for topic in category.topic_list.topics {
        let topic_detail = client.fetch_topic(topic.id, true)?;
        let raw = topic_detail
            .post_stream
            .posts
            .first()
            .and_then(|p| p.raw.clone())
            .unwrap_or_default();
        let filename = format!("{}.md", slugify(&topic.title));
        let contents = render_category_topic(&topic, &discourse.baseurl, &raw);
        write_markdown(&dir.join(filename), &contents)?;
    }
    println!("{}", dir.display());
    Ok(())
}

/// Render one pulled topic as a Markdown document with YAML front matter
/// (`title` / `topic_id` / `url` / `pulled_at`) followed by the OP's raw body.
///
/// The `topic_id` is the durable binding `category push` routes on, so edits
/// to the title or filename no longer risk creating a duplicate topic. Mirrors
/// the front matter written by `topic pull --full`.
fn render_category_topic(topic: &TopicSummary, baseurl: &str, raw: &str) -> String {
    let base = normalize_baseurl(baseurl);
    let url = format!("{}/t/{}/{}", base, topic.slug, topic.id);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("title: {}\n", yaml_scalar(&topic.title)));
    out.push_str(&format!("topic_id: {}\n", topic.id));
    out.push_str(&format!("url: {}\n", url));
    out.push_str(&format!("pulled_at: {}\n", current_utc_iso8601()));
    out.push_str("---\n\n");
    out.push_str(raw.trim_end());
    out.push('\n');
    out
}

/// One planned change for `category push`, decided before any mutation so
/// the whole plan can be printed and reviewed up front. This satisfies the
/// governance requirement driving the spec: never push without a reviewable
/// plan, never create a topic without deliberate intent.
enum PushAction {
    /// Body differs from the remote OP; update post `post_id`.
    Update {
        path: PathBuf,
        topic_id: u64,
        post_id: u64,
        body: String,
    },
    /// Body matches the remote OP (modulo trailing whitespace); no write.
    Unchanged { path: PathBuf, topic_id: u64 },
    /// No remote match; create a new topic (skipped under `--updates-only`).
    Create {
        path: PathBuf,
        title: String,
        body: String,
    },
}

pub fn category_push(
    config: &Config,
    discourse_name: &str,
    category: &str,
    local_path: &Path,
    dry_run: bool,
    updates_only: bool,
    edit_opts: PostEditOptions,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let category_id = resolve_category_id(&client, category)?;
    let existing = client.fetch_category(category_id)?;
    let topics = existing.topic_list.topics;

    // Collect Markdown files in a stable order so the plan is deterministic.
    let mut paths: Vec<PathBuf> = fs::read_dir(local_path)
        .with_context(|| format!("reading {}", local_path.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    paths.sort();

    // Plan first: decide every action (and fail any --updates-only mismatch)
    // before mutating anything on the server.
    let mut plan = Vec::with_capacity(paths.len());
    for path in paths {
        let raw = read_markdown(&path)?;
        let (front, body) = strip_frontmatter(&raw);
        let title = front
            .get("title")
            .cloned()
            .or_else(|| extract_title(&body))
            .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().to_string());

        let topic_id = route_topic_id(&front, &title, &path, &topics);

        match topic_id {
            Some(id) => {
                let detail = client.fetch_topic(id, true)?;
                let post = detail
                    .post_stream
                    .posts
                    .first()
                    .ok_or_else(|| anyhow!("topic {} has no posts", id))?;
                let remote = post.raw.as_deref().unwrap_or_default();
                if remote.trim_end() == body.trim_end() {
                    plan.push(PushAction::Unchanged { path, topic_id: id });
                } else {
                    plan.push(PushAction::Update {
                        path,
                        topic_id: id,
                        post_id: post.id,
                        body,
                    });
                }
            }
            None => {
                if updates_only {
                    return Err(anyhow!(
                        "no matching topic for {} (title: {:?})\n\
                         hint: remove --updates-only to allow new topic creation, \
                         or check the filename/topic_id matches an existing topic",
                        path.display(),
                        title
                    ));
                }
                plan.push(PushAction::Create { path, title, body });
            }
        }
    }

    print_push_plan(&plan, &discourse.name, category_id, dry_run);

    if !dry_run {
        for action in &plan {
            match action {
                PushAction::Update { post_id, body, .. } => {
                    client.update_post(*post_id, body, edit_opts)?;
                }
                PushAction::Create { title, body, .. } => {
                    client.create_topic(category_id, title, body)?;
                }
                PushAction::Unchanged { .. } => {}
            }
        }
    }

    Ok(())
}

/// Print the push plan using the same `~` (change) / `+` (create) /
/// `=` (unchanged) sigils as `setting push`, prefixed with `[dry-run]`
/// when nothing will actually be written.
fn print_push_plan(plan: &[PushAction], discourse: &str, category_id: u64, dry_run: bool) {
    let prefix = if dry_run { "[dry-run] " } else { "" };
    let updates = plan
        .iter()
        .filter(|a| matches!(a, PushAction::Update { .. }))
        .count();
    let creates = plan
        .iter()
        .filter(|a| matches!(a, PushAction::Create { .. }))
        .count();
    let unchanged = plan
        .iter()
        .filter(|a| matches!(a, PushAction::Unchanged { .. }))
        .count();

    println!(
        "{}Category push plan for {} (category {}): {} update{}, {} create{}, {} unchanged",
        prefix,
        discourse,
        category_id,
        updates,
        if updates == 1 { "" } else { "s" },
        creates,
        if creates == 1 { "" } else { "s" },
        unchanged,
    );
    for action in plan {
        match action {
            PushAction::Update {
                path,
                topic_id,
                body,
                ..
            } => {
                println!(
                    "  ~ {} → topic {} ({} bytes)",
                    file_label(path),
                    topic_id,
                    body.len()
                );
            }
            PushAction::Unchanged { path, topic_id } => {
                println!("  = {} (topic {}, unchanged)", file_label(path), topic_id);
            }
            PushAction::Create { path, title, body } => {
                println!(
                    "  + {} → new topic \"{}\" ({} bytes)",
                    file_label(path),
                    title,
                    body.len()
                );
            }
        }
    }
}

/// File name for plan output, falling back to the full path if there is none.
fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn resolve_category_id(client: &DiscourseClient, category: &str) -> Result<u64> {
    if let Ok(id) = category.parse::<u64>() {
        return Ok(id);
    }
    let slug = category.trim();
    if slug.is_empty() {
        return Err(anyhow!(
            "missing category identifier for category operation"
        ));
    }
    let categories = client.fetch_categories()?;
    let category = categories
        .into_iter()
        .find(|cat| cat.slug == slug)
        .ok_or_else(|| not_found("category", slug))?;
    category.id.ok_or_else(|| not_found("category", slug))
}

fn flatten_categories(category: &CategoryInfo, out: &mut Vec<CategoryInfo>) {
    out.push(category.clone());
    for sub in &category.subcategory_list {
        flatten_categories(sub, out);
    }
}

fn unique_categories(flat: Vec<CategoryInfo>) -> Vec<CategoryInfo> {
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();
    for category in flat {
        if let Some(id) = category.id
            && !seen.insert(id)
        {
            continue;
        }
        unique.push(category);
    }
    unique
}

fn print_category_tree(categories: &[CategoryInfo]) {
    let mut ordered_ids = Vec::new();
    let mut map = std::collections::HashMap::new();
    for category in categories {
        if let Some(id) = category.id {
            map.entry(id).or_insert_with(|| {
                ordered_ids.push(id);
                category.clone()
            });
        }
    }

    let mut children: std::collections::HashMap<u64, Vec<u64>> = std::collections::HashMap::new();
    for category in map.values() {
        if let (Some(id), Some(parent_id)) = (category.id, category.parent_category_id)
            && map.contains_key(&parent_id)
        {
            let entry = children.entry(parent_id).or_default();
            if !entry.contains(&id) {
                entry.push(id);
            }
        }
    }

    let mut roots = Vec::new();
    for id in &ordered_ids {
        if let Some(category) = map.get(id) {
            match category.parent_category_id {
                Some(parent_id) if map.contains_key(&parent_id) => {}
                _ => roots.push(*id),
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    let last_index = roots.len().saturating_sub(1);
    for (idx, id) in roots.into_iter().enumerate() {
        let is_last = idx == last_index;
        print_category_node(&map, &children, id, "", is_last, &mut seen);
    }
}

fn print_category_node(
    map: &std::collections::HashMap<u64, CategoryInfo>,
    children: &std::collections::HashMap<u64, Vec<u64>>,
    id: u64,
    prefix: &str,
    is_last: bool,
    seen: &mut std::collections::HashSet<u64>,
) {
    if !seen.insert(id) {
        return;
    }
    if let Some(category) = map.get(&id) {
        let branch = if is_last {
            "└── ".to_string()
        } else {
            "├── ".to_string()
        };
        println!("{}{}{} - {}", prefix, branch, id, category.name);
        if let Some(child_ids) = children.get(&id) {
            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            let last_index = child_ids.len().saturating_sub(1);
            for (idx, child_id) in child_ids.iter().enumerate() {
                let child_last = idx == last_index;
                print_category_node(map, children, *child_id, &new_prefix, child_last, seen);
            }
        }
    }
}

fn extract_title(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(title) = line.strip_prefix("# ") {
            return Some(title.trim().to_string());
        }
        break;
    }
    None
}

/// Resolve which remote topic a local file targets. A `topic_id` from front
/// matter wins outright (the durable binding written by `category pull`);
/// otherwise fall back to slug/title matching for pre-front-matter snapshots.
/// `None` means "no remote match" — which `category push` turns into a create
/// (or, under `--updates-only`, an error).
fn route_topic_id(
    front: &std::collections::HashMap<String, String>,
    title: &str,
    path: &Path,
    topics: &[TopicSummary],
) -> Option<u64> {
    front
        .get("topic_id")
        .and_then(|s| s.trim().parse::<u64>().ok())
        .or_else(|| find_topic_match(topics, title, path).map(|t| t.id))
}

fn find_topic_match<'a>(
    topics: &'a [TopicSummary],
    title: &str,
    path: &Path,
) -> Option<&'a TopicSummary> {
    let slug = slugify(title);
    topics.iter().find(|topic| {
        topic.slug == slug
            || topic.title.eq_ignore_ascii_case(title)
            || path
                .file_stem()
                .map(|s| s.to_string_lossy().eq_ignore_ascii_case(&topic.slug))
                .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn summary(id: u64, title: &str, slug: &str) -> TopicSummary {
        TopicSummary {
            id,
            title: title.to_string(),
            slug: slug.to_string(),
        }
    }

    fn front(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn render_category_topic_emits_front_matter() {
        let topic = summary(412, "Dependency management", "dependency-management");
        let out = render_category_topic(&topic, "https://forum.rcpch.tech/", "Body here.\n");
        assert!(out.starts_with("---\n"));
        assert!(out.contains("title: Dependency management\n"));
        assert!(out.contains("topic_id: 412\n"));
        assert!(out.contains("url: https://forum.rcpch.tech/t/dependency-management/412\n"));
        assert!(out.contains("pulled_at: "));
        assert!(out.contains("\n---\n\nBody here.\n"));
    }

    #[test]
    fn render_then_strip_round_trips_the_body() {
        let topic = summary(7, "Intro: getting started", "intro-getting-started");
        let body = "First paragraph.\n\n---\n\nSecond, after a rule.\n";
        let rendered = render_category_topic(&topic, "https://x.test", body);
        let (front, recovered) = strip_frontmatter(&rendered);
        assert_eq!(front.get("topic_id").map(String::as_str), Some("7"));
        // Title carrying a colon is round-tripped through YAML quoting.
        assert_eq!(
            front.get("title").map(String::as_str),
            Some("Intro: getting started")
        );
        assert_eq!(recovered, body);
    }

    #[test]
    fn route_prefers_front_matter_topic_id_over_slug() {
        // The slug would match topic 1, but the front-matter id wins — this is
        // the whole point of Gap 1: a renamed/retitled file still routes home.
        let topics = vec![summary(1, "Old Title", "renamed-file")];
        let f = front(&[("topic_id", "412")]);
        let path = PathBuf::from("renamed-file.md");
        assert_eq!(route_topic_id(&f, "New Title", &path, &topics), Some(412));
    }

    #[test]
    fn route_falls_back_to_slug_match_without_front_matter() {
        let topics = vec![summary(
            55,
            "Dependency management",
            "dependency-management",
        )];
        let f = HashMap::new();
        let path = PathBuf::from("dependency-management.md");
        assert_eq!(
            route_topic_id(&f, "Dependency management", &path, &topics),
            Some(55)
        );
    }

    #[test]
    fn route_returns_none_when_nothing_matches() {
        let topics = vec![summary(
            55,
            "Dependency management",
            "dependency-management",
        )];
        let f = HashMap::new();
        let path = PathBuf::from("brand-new-file.md");
        assert_eq!(route_topic_id(&f, "Brand new file", &path, &topics), None);
    }

    #[test]
    fn route_ignores_unparseable_front_matter_topic_id() {
        // A garbage topic_id must not route; fall through to slug matching.
        let topics = vec![summary(
            55,
            "Dependency management",
            "dependency-management",
        )];
        let f = front(&[("topic_id", "not-a-number")]);
        let path = PathBuf::from("dependency-management.md");
        assert_eq!(
            route_topic_id(&f, "Dependency management", &path, &topics),
            Some(55)
        );
    }
}
