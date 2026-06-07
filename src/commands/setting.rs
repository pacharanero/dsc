use crate::api::{DiscourseClient, SiteSettingDetail};
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, parse_tags, select_discourse};
use crate::config::{Config, DiscourseConfig};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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

// ─── Pull (snapshot) ──────────────────────────────────────────────────────────

/// On-disk settings snapshot file (schema version 1).
///
/// Spec: `spec/setting-sync.md`. The file is self-documenting: it carries
/// each setting's default, type, category, and description so that a human
/// (or LLM) reading the file can understand each entry without consulting
/// the API. On `push`, only `name` and `value` are honoured.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsFile {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discourse_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pulled_at: Option<String>,
    #[serde(default)]
    pub settings: Vec<SettingsEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsEntry {
    pub name: String,
    pub value: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Renamed to avoid the Rust keyword. Serializes as `type`.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub setting_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Settings excluded from `pull` because they are computed/read-only on the
/// server. Keep the list small; unknown read-only settings are handled
/// gracefully by `push` (server returns 422, we warn and continue).
const READONLY_SETTINGS: &[&str] = &[];

/// Pull all site settings to a local file.
pub fn pull_settings(
    config: &Config,
    discourse_name: &str,
    local_path: &Path,
    changed_only: bool,
    category: Option<&str>,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let server = client.list_site_settings_detailed()?;
    let discourse_version = client.fetch_version().ok().flatten();

    let mut entries: Vec<SettingsEntry> = server
        .into_iter()
        .filter(|s| !READONLY_SETTINGS.contains(&s.setting.as_str()))
        .filter(|s| match category {
            Some(cat) => s.category.eq_ignore_ascii_case(cat),
            None => true,
        })
        .filter(|s| {
            if !changed_only {
                return true;
            }
            !values_equal(&s.value, &s.default)
        })
        .map(detail_to_entry)
        .collect();

    // Sort by category, then by name for stable diffs.
    entries.sort_by(|a, b| {
        let ca = a.category.as_deref().unwrap_or("");
        let cb = b.category.as_deref().unwrap_or("");
        ca.cmp(cb).then_with(|| a.name.cmp(&b.name))
    });

    let pulled_at = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let file = SettingsFile {
        version: 1,
        discourse_version,
        pulled_at: Some(pulled_at),
        settings: entries,
    };

    let content = if is_json_path(local_path) {
        serde_json::to_string_pretty(&file).context("serializing settings as JSON")?
    } else {
        serde_yaml::to_string(&file).context("serializing settings as YAML")?
    };

    fs::write(local_path, &content)
        .with_context(|| format!("writing {}", local_path.display()))?;

    println!(
        "Wrote {} setting{} to {}",
        file.settings.len(),
        if file.settings.len() == 1 { "" } else { "s" },
        local_path.display()
    );
    Ok(())
}

fn detail_to_entry(d: SiteSettingDetail) -> SettingsEntry {
    SettingsEntry {
        name: d.setting,
        value: d.value,
        default: if d.default.is_null() {
            None
        } else {
            Some(d.default)
        },
        setting_type: empty_to_none(d.setting_type),
        category: empty_to_none(d.category),
        description: empty_to_none(d.description),
    }
}

fn empty_to_none(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn values_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    // Discourse returns numeric and boolean settings as JSON values; comparing
    // the parsed Value handles all simple types. For string-typed values that
    // happen to differ only in whitespace, leave the strict comparison; users
    // who hit edge cases can edit the snapshot directly.
    a == b
}

fn is_json_path(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}
