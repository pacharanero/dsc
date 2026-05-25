use crate::api::{DiscourseClient, TagGroupInfo};
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, select_discourse};
use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub fn tag_list(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let mut tags = client.list_tags()?;
    tags.sort_by(|a, b| a.text.cmp(&b.text));

    match format {
        ListFormat::Text => {
            if tags.is_empty() {
                println!("No tags.");
                return Ok(());
            }
            let name_width = tags
                .iter()
                .map(|t| t.text.len())
                .max()
                .unwrap_or(0)
                .max(4);
            for tag in &tags {
                println!(
                    "{:<width$}  {}",
                    tag.text,
                    tag.count,
                    width = name_width
                );
            }
        }
        ListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&tags)?);
        }
        ListFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&tags)?);
        }
    }

    Ok(())
}

pub fn tag_apply(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    tag: &str,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let current = client.fetch_topic_tags(topic_id)?;
    let Some(next) = next_tags_after_apply(&current, tag) else {
        println!("Topic {} already tagged '{}'", topic_id, tag);
        return Ok(());
    };
    if dry_run {
        println!(
            "[dry-run] would set tags on topic {} to: [{}]",
            topic_id,
            next.join(", ")
        );
        return Ok(());
    }
    let after = client.set_topic_tags(topic_id, &next)?;
    println!("Topic {} tags: [{}]", topic_id, after.join(", "));
    Ok(())
}

/// Compute the resulting tag list when adding `tag` to `current`. Returns
/// None when the tag is already present.
fn next_tags_after_apply(current: &[String], tag: &str) -> Option<Vec<String>> {
    if current.iter().any(|t| t == tag) {
        return None;
    }
    let mut next = current.to_vec();
    next.push(tag.to_string());
    Some(next)
}

/// Compute the resulting tag list when removing `tag` from `current`. Returns
/// None when the tag is not present.
fn next_tags_after_remove(current: &[String], tag: &str) -> Option<Vec<String>> {
    if !current.iter().any(|t| t == tag) {
        return None;
    }
    Some(current.iter().filter(|t| *t != tag).cloned().collect())
}

pub fn tag_remove(
    config: &Config,
    discourse_name: &str,
    topic_id: u64,
    tag: &str,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let current = client.fetch_topic_tags(topic_id)?;
    let Some(next) = next_tags_after_remove(&current, tag) else {
        println!("Topic {} does not have tag '{}'", topic_id, tag);
        return Ok(());
    };
    if dry_run {
        println!(
            "[dry-run] would set tags on topic {} to: [{}]",
            topic_id,
            next.join(", ")
        );
        return Ok(());
    }
    let after = client.set_topic_tags(topic_id, &next)?;
    println!("Topic {} tags: [{}]", topic_id, after.join(", "));
    Ok(())
}

// ─── Taxonomy file schema ─────────────────────────────────────────────────────

/// The on-disk taxonomy file (version 1).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaxonomyFile {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<TagEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tag_groups: Vec<TagGroupEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagGroupEntry {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub one_per_topic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn is_false(v: &bool) -> bool {
    !v
}

// ─── Pull ─────────────────────────────────────────────────────────────────────

pub fn tag_pull(
    config: &Config,
    discourse_name: &str,
    local_path: &Path,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let server_tags = client.list_tags()?;

    // Collect tag entries with descriptions
    let mut tag_entries: Vec<TagEntry> = Vec::new();
    for t in &server_tags {
        let description = client.get_tag_description(&t.text).unwrap_or(None);
        tag_entries.push(TagEntry {
            name: t.text.clone(),
            description,
        });
    }
    tag_entries.sort_by(|a, b| a.name.cmp(&b.name));

    // Attempt tag groups (admin-only)
    let tag_groups = match client.list_tag_groups()? {
        Some(groups) => {
            let mut entries: Vec<TagGroupEntry> = groups
                .into_iter()
                .map(|g| {
                    let permissions = g.permissions.and_then(|p| {
                        // Discourse returns permissions as {"group_name": "level_int"}
                        // or a more complex structure; normalize to group→level string
                        parse_tag_group_permissions(&p)
                    });
                    let mut tags = g.tag_names;
                    tags.sort();
                    TagGroupEntry {
                        name: g.name,
                        description: None, // not returned by list endpoint
                        one_per_topic: g.one_per_topic,
                        parent_tag: g.parent_tag_name,
                        permissions,
                        tags,
                    }
                })
                .collect();
            entries.sort_by(|a, b| a.name.cmp(&b.name));
            entries
        }
        None => {
            eprintln!("Warning: tag groups not accessible (requires admin API key); omitting from output.");
            Vec::new()
        }
    };

    let taxonomy = TaxonomyFile {
        version: 1,
        tags: tag_entries,
        tag_groups,
    };

    let content = if is_json_path(local_path) {
        serde_json::to_string_pretty(&taxonomy).context("serializing taxonomy as JSON")?
    } else {
        serde_yaml::to_string(&taxonomy).context("serializing taxonomy as YAML")?
    };

    fs::write(local_path, &content)
        .with_context(|| format!("writing {}", local_path.display()))?;
    println!("Wrote taxonomy to {}", local_path.display());
    Ok(())
}

fn parse_tag_group_permissions(value: &serde_json::Value) -> Option<BTreeMap<String, String>> {
    // Discourse API returns permissions as: {"everyone": 1} where 1=full, 3=readonly
    // or as an object. Normalize to string labels.
    let obj = value.as_object()?;
    if obj.is_empty() {
        return None;
    }
    let mut map = BTreeMap::new();
    for (group, level) in obj {
        let level_str = match level.as_u64() {
            Some(1) => "full".to_string(),
            Some(3) => "readonly".to_string(),
            Some(n) => n.to_string(),
            None => level.as_str().unwrap_or("full").to_string(),
        };
        map.insert(group.clone(), level_str);
    }
    Some(map)
}

fn is_json_path(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

// ─── Push ─────────────────────────────────────────────────────────────────────

pub fn tag_push(
    config: &Config,
    discourse_name: &str,
    local_path: &Path,
    prune: bool,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let content = fs::read_to_string(local_path)
        .with_context(|| format!("reading {}", local_path.display()))?;
    let taxonomy: TaxonomyFile = if is_json_path(local_path) {
        serde_json::from_str(&content).context("parsing taxonomy JSON")?
    } else {
        serde_yaml::from_str(&content).context("parsing taxonomy YAML")?
    };

    if taxonomy.version != 1 {
        anyhow::bail!("unsupported taxonomy file version: {}", taxonomy.version);
    }

    // Desired tag set = explicit tags + all tags mentioned in groups
    let desired_tags: BTreeSet<String> = taxonomy
        .tags
        .iter()
        .map(|t| t.name.clone())
        .chain(taxonomy.tag_groups.iter().flat_map(|g| g.tags.clone()))
        .collect();

    // Build description map from explicit entries
    let desired_descriptions: BTreeMap<String, Option<String>> = taxonomy
        .tags
        .iter()
        .map(|t| (t.name.clone(), t.description.clone()))
        .collect();

    // ── Reconcile tags ────────────────────────────────────────────────────────
    let server_tags = client.list_tags()?;
    let server_tag_names: BTreeSet<String> = server_tags.iter().map(|t| t.text.clone()).collect();

    let tags_to_create: Vec<&String> = desired_tags.difference(&server_tag_names).collect();
    let tags_to_delete: Vec<&String> = if prune {
        server_tag_names.difference(&desired_tags).collect()
    } else {
        Vec::new()
    };

    // Tags that exist and have a description to set
    let tags_to_update: Vec<(&String, &Option<String>)> = desired_descriptions
        .iter()
        .filter(|(name, desc)| server_tag_names.contains(*name) && desc.is_some())
        .collect();

    if dry_run {
        println!("[dry-run] Tag plan:");
        if tags_to_create.is_empty() && tags_to_delete.is_empty() && tags_to_update.is_empty() {
            println!("  (no tag changes)");
        }
        for name in &tags_to_create {
            println!("  + create tag: {}", name);
        }
        for (name, desc) in &tags_to_update {
            println!("  ~ update tag: {} (set description: {:?})", name, desc.as_deref().unwrap_or(""));
        }
        for name in &tags_to_delete {
            println!("  - delete tag: {}", name);
        }
    } else {
        for name in &tags_to_create {
            let desc = desired_descriptions.get(*name).and_then(|d| d.as_deref());
            client.update_tag(name, desc)
                .with_context(|| format!("creating/updating tag '{}'", name))?;
            println!("  + created tag: {}", name);
        }
        for (name, desc) in &tags_to_update {
            client.update_tag(name, desc.as_deref())
                .with_context(|| format!("updating tag '{}'", name))?;
            println!("  ~ updated tag: {}", name);
        }
        for name in &tags_to_delete {
            client.delete_tag(name)
                .with_context(|| format!("deleting tag '{}'", name))?;
            println!("  - deleted tag: {}", name);
        }
    }

    // ── Reconcile tag groups ──────────────────────────────────────────────────
    let server_groups = match client.list_tag_groups()? {
        Some(groups) => groups,
        None => {
            if !taxonomy.tag_groups.is_empty() {
                eprintln!("Warning: tag groups not accessible (requires admin API key); skipping group reconciliation.");
            }
            return Ok(());
        }
    };

    let server_groups_by_name: BTreeMap<String, &TagGroupInfo> = server_groups
        .iter()
        .map(|g| (g.name.clone(), g))
        .collect();

    let desired_group_names: BTreeSet<String> =
        taxonomy.tag_groups.iter().map(|g| g.name.clone()).collect();
    let server_group_names: BTreeSet<String> =
        server_groups.iter().map(|g| g.name.clone()).collect();

    let groups_to_create: Vec<&TagGroupEntry> = taxonomy
        .tag_groups
        .iter()
        .filter(|g| !server_group_names.contains(&g.name))
        .collect();

    let groups_to_update: Vec<(&TagGroupEntry, u64)> = taxonomy
        .tag_groups
        .iter()
        .filter_map(|g| {
            server_groups_by_name
                .get(&g.name)
                .map(|sg| (g, sg.id))
        })
        .filter(|(desired, _id)| {
            // Only update if something differs
            let server = server_groups_by_name.get(&desired.name).unwrap();
            let mut server_tags = server.tag_names.clone();
            server_tags.sort();
            let mut desired_tags = desired.tags.clone();
            desired_tags.sort();
            server_tags != desired_tags
                || server.one_per_topic != desired.one_per_topic
                || server.parent_tag_name != desired.parent_tag
        })
        .collect();

    let groups_to_delete: Vec<(&str, u64)> = if prune {
        server_groups
            .iter()
            .filter(|g| !desired_group_names.contains(&g.name))
            .map(|g| (g.name.as_str(), g.id))
            .collect()
    } else {
        Vec::new()
    };

    if dry_run {
        println!("[dry-run] Tag group plan:");
        if groups_to_create.is_empty() && groups_to_update.is_empty() && groups_to_delete.is_empty()
        {
            println!("  (no group changes)");
        }
        for g in &groups_to_create {
            println!("  + create group: {} (tags: [{}])", g.name, g.tags.join(", "));
        }
        for (g, _id) in &groups_to_update {
            println!("  ~ update group: {} (tags: [{}])", g.name, g.tags.join(", "));
        }
        for (name, _id) in &groups_to_delete {
            println!("  - delete group: {}", name);
        }
    } else {
        for g in &groups_to_create {
            let payload = build_tag_group_payload(g);
            client.create_tag_group(&payload)
                .with_context(|| format!("creating tag group '{}'", g.name))?;
            println!("  + created group: {}", g.name);
        }
        for (g, id) in &groups_to_update {
            let payload = build_tag_group_payload(g);
            client.update_tag_group(*id, &payload)
                .with_context(|| format!("updating tag group '{}'", g.name))?;
            println!("  ~ updated group: {}", g.name);
        }
        for (name, id) in &groups_to_delete {
            client.delete_tag_group(*id)
                .with_context(|| format!("deleting tag group '{}'", name))?;
            println!("  - deleted group: {}", name);
        }
    }

    if dry_run {
        println!("[dry-run] No changes applied.");
    } else {
        println!("Push complete.");
    }
    Ok(())
}

fn build_tag_group_payload(entry: &TagGroupEntry) -> serde_json::Value {
    let mut group = serde_json::Map::new();
    group.insert("name".to_string(), serde_json::json!(entry.name));
    group.insert("tag_names".to_string(), serde_json::json!(entry.tags));
    group.insert("one_per_topic".to_string(), serde_json::json!(entry.one_per_topic));
    if let Some(parent) = &entry.parent_tag {
        group.insert("parent_tag_name".to_string(), serde_json::json!([parent]));
    }
    if let Some(perms) = &entry.permissions {
        let perm_map: BTreeMap<&String, u64> = perms
            .iter()
            .map(|(k, v)| {
                let level = match v.as_str() {
                    "full" => 1,
                    "readonly" => 3,
                    _ => v.parse().unwrap_or(1),
                };
                (k, level)
            })
            .collect();
        group.insert("permissions".to_string(), serde_json::json!(perm_map));
    }
    serde_json::json!({ "tag_group": group })
}

#[cfg(test)]
mod tests {
    use super::{next_tags_after_apply, next_tags_after_remove};

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn apply_adds_when_absent() {
        let got = next_tags_after_apply(&s(&["foo", "bar"]), "baz").unwrap();
        assert_eq!(got, s(&["foo", "bar", "baz"]));
    }

    #[test]
    fn apply_returns_none_when_already_present() {
        assert!(next_tags_after_apply(&s(&["foo", "bar"]), "foo").is_none());
    }

    #[test]
    fn apply_to_empty_list_works() {
        let got = next_tags_after_apply(&s(&[]), "first").unwrap();
        assert_eq!(got, s(&["first"]));
    }

    #[test]
    fn remove_drops_present_tag() {
        let got = next_tags_after_remove(&s(&["foo", "bar", "baz"]), "bar").unwrap();
        assert_eq!(got, s(&["foo", "baz"]));
    }

    #[test]
    fn remove_returns_none_when_absent() {
        assert!(next_tags_after_remove(&s(&["foo", "bar"]), "baz").is_none());
    }

    #[test]
    fn remove_last_tag_leaves_empty_list() {
        let got = next_tags_after_remove(&s(&["only"]), "only").unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn apply_is_case_sensitive() {
        // Discourse tags are lowercase canonically, but we don't normalize —
        // the API returns and accepts whatever is sent. Document the behaviour.
        let got = next_tags_after_apply(&s(&["Foo"]), "foo").unwrap();
        assert_eq!(got, s(&["Foo", "foo"]));
    }
}
