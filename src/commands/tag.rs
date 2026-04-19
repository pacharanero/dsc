use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, select_discourse};
use crate::config::Config;
use anyhow::Result;

pub fn tag_list(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let mut tags = client.list_tags()?;
    tags.sort_by(|a, b| a.id.cmp(&b.id));

    match format {
        ListFormat::Text => {
            if tags.is_empty() {
                println!("No tags.");
                return Ok(());
            }
            let name_width = tags
                .iter()
                .map(|t| t.id.len())
                .max()
                .unwrap_or(0)
                .max(4);
            for tag in &tags {
                println!(
                    "{:<width$}  {}",
                    tag.id,
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
