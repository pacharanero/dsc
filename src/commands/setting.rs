use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, parse_tags, select_discourse};
use crate::config::{Config, DiscourseConfig};
use anyhow::{anyhow, Result};
use serde::Serialize;

/// Set a site setting. If `discourse_name` is given, only that discourse is updated.
/// Otherwise all discourses matching `tags` are updated.
pub fn set_site_setting(
    config: &Config,
    discourse_name: Option<&str>,
    setting: &str,
    value: &str,
    tags: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    if let Some(name) = discourse_name {
        let discourse = select_discourse(config, Some(name))?;
        ensure_api_credentials(discourse)?;
        if dry_run {
            println!(
                "[dry-run] {}: would set {} = {}",
                discourse.name, setting, value
            );
            return Ok(());
        }
        let client = DiscourseClient::new(discourse)?;
        client.update_site_setting(setting, value)?;
        println!("{}: updated {}", discourse.name, setting);
        return Ok(());
    }

    // No specific discourse - use tag filter across all discourses.
    let filter = tags.map(parse_tags).unwrap_or_default();
    let matches_filter = |disc: &DiscourseConfig| {
        if filter.is_empty() {
            return true;
        }
        let disc_tags = disc.tags.as_ref().map(|t| {
            t.iter()
                .map(|tag| tag.to_ascii_lowercase())
                .collect::<Vec<_>>()
        });
        let Some(disc_tags) = disc_tags else {
            return false;
        };
        filter.iter().any(|tag| {
            let tag = tag.to_ascii_lowercase();
            disc_tags.iter().any(|t| t == &tag)
        })
    };

    let mut matched = 0;
    for discourse in config.discourse.iter().filter(|d| matches_filter(d)) {
        matched += 1;
        ensure_api_credentials(discourse)?;
        if dry_run {
            println!(
                "[dry-run] {}: would set {} = {}",
                discourse.name, setting, value
            );
            continue;
        }
        let client = DiscourseClient::new(discourse)?;
        client.update_site_setting(setting, value)?;
        println!("{}: updated {}", discourse.name, setting);
    }

    if matched == 0 {
        return Err(anyhow!("no discourses matched the tag filter"));
    }

    Ok(())
}

/// Get the current value of a single site setting.
pub fn get_site_setting(config: &Config, discourse_name: &str, setting: &str) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let value = client.fetch_site_setting(setting)?;
    println!("{}", value);
    Ok(())
}

#[derive(Debug, Serialize)]
struct SettingEntry {
    setting: String,
    value: String,
    category: String,
}

/// List all site settings.
pub fn list_site_settings(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
    verbose: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let raw = client.list_site_settings()?;

    let settings_arr = raw
        .get("site_settings")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let entries: Vec<SettingEntry> = settings_arr
        .into_iter()
        .map(|entry| {
            let setting = entry
                .get("setting")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let value = match entry
                .get("value")
                .cloned()
                .unwrap_or(serde_json::Value::Null)
            {
                serde_json::Value::String(s) => s,
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            let category = entry
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("uncategorized")
                .to_string();
            SettingEntry {
                setting,
                value,
                category,
            }
        })
        .collect();

    match format {
        ListFormat::Text => {
            if entries.is_empty() && !verbose {
                println!("No settings found.");
                return Ok(());
            }
            for e in &entries {
                println!("{} = {}", e.setting, e.value);
            }
        }
        ListFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        ListFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&entries)?);
        }
    }

    Ok(())
}
