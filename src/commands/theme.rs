use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, not_found, select_discourse};
use crate::commands::update::run_ssh_command;
use crate::config::{Config, DiscourseConfig};
use crate::utils::slugify;
use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use serde_json::{Value, json};
use std::path::Path;

#[derive(Debug, Serialize)]
struct ThemeListEntry {
    id: u64,
    name: String,
    status: String,
}

pub fn theme_list(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
    verbose: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let response = client.list_themes()?;
    let themes = response
        .get("themes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let entries: Vec<ThemeListEntry> = themes
        .into_iter()
        .map(|theme| {
            let id = theme.get("id").and_then(|v| v.as_u64()).unwrap_or_default();
            let name = theme
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let status = theme
                .get("enabled")
                .and_then(|v| v.as_bool())
                .map(|value| {
                    if value {
                        "enabled".to_string()
                    } else {
                        "disabled".to_string()
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());
            ThemeListEntry { id, name, status }
        })
        .collect();

    match format {
        ListFormat::Text => {
            if entries.is_empty() && !verbose {
                println!("No themes found.");
                return Ok(());
            }
            for theme in entries {
                println!("{} - {} - {}", theme.id, theme.name, theme.status);
            }
        }
        ListFormat::Json => {
            let raw = serde_json::to_string_pretty(&entries)?;
            println!("{}", raw);
        }
        ListFormat::Yaml => {
            let raw = serde_yaml::to_string(&entries)?;
            println!("{}", raw);
        }
    }
    Ok(())
}

pub fn theme_install(
    config: &Config,
    discourse_name: &str,
    url: &str,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    let target = ssh_target(discourse);
    let template = std::env::var("DSC_SSH_THEME_INSTALL_CMD")
        .map_err(|_| {
            anyhow!(
                "missing DSC_SSH_THEME_INSTALL_CMD for theme install; set DSC_SSH_THEME_INSTALL_CMD to your install command"
            )
        })?;
    let command = render_template(&template, &[("url", url), ("name", url)]);
    if dry_run {
        println!("[dry-run] would run on {}: {}", target, command);
        return Ok(());
    }
    let output = run_ssh_command(&target, &command)?;
    println!("Theme install completed: {}", url);
    if !output.trim().is_empty() {
        println!("{}", output.trim());
    }
    Ok(())
}

pub fn theme_remove(
    config: &Config,
    discourse_name: &str,
    name: &str,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    let target = ssh_target(discourse);
    let template = std::env::var("DSC_SSH_THEME_REMOVE_CMD")
        .map_err(|_| {
            anyhow!(
                "missing DSC_SSH_THEME_REMOVE_CMD for theme remove; set DSC_SSH_THEME_REMOVE_CMD to your remove command"
            )
        })?;
    let command = render_template(&template, &[("name", name), ("url", name)]);
    if dry_run {
        println!("[dry-run] would run on {}: {}", target, command);
        return Ok(());
    }
    let output = run_ssh_command(&target, &command)?;
    println!("Theme removal completed: {}", name);
    if !output.trim().is_empty() {
        println!("{}", output.trim());
    }
    Ok(())
}

/// Pull a theme to a local JSON file.
pub fn theme_pull(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
    local_path: Option<&Path>,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let response = client.fetch_theme(theme_id)?;

    // Unwrap {"theme": {...}} envelope if present
    let theme = response.get("theme").unwrap_or(&response);

    let path = match local_path {
        Some(p) => p.to_path_buf(),
        None => {
            let name_slug = theme
                .get("name")
                .and_then(|v| v.as_str())
                .map(slugify)
                .unwrap_or_else(|| format!("theme-{}", theme_id));
            let filename = format!("{}.json", name_slug);
            std::env::current_dir()
                .context("getting current directory")?
                .join(filename)
        }
    };

    let content =
        serde_json::to_string_pretty(theme).context("serializing theme to JSON")?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
    }
    std::fs::write(&path, content)
        .with_context(|| format!("writing {}", path.display()))?;
    println!("{}", path.display());
    Ok(())
}

/// Push a local JSON file to create or update a theme.
pub fn theme_push(
    config: &Config,
    discourse_name: &str,
    json_path: &Path,
    theme_id: Option<u64>,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let raw = std::fs::read_to_string(json_path)
        .with_context(|| format!("reading {}", json_path.display()))?;
    let parsed: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing JSON from {}", json_path.display()))?;

    // Unwrap {"theme": {...}} envelope if present
    let theme = if let Some(inner) = parsed.get("theme") {
        inner.clone()
    } else {
        parsed
    };

    let push_data = build_push_payload(&theme);

    let target_id = theme_id.or_else(|| theme.get("id").and_then(|v| v.as_u64()));

    if let Some(id) = target_id {
        client.update_theme(id, &push_data)?;
        println!("{}", id);
    } else {
        if push_data
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
        {
            return Err(anyhow!(
                "missing name in theme file; set name or pass a theme ID to update"
            ));
        }
        let new_id = client.create_theme(&push_data)?;
        println!("{}", new_id);
    }

    Ok(())
}

/// Duplicate a theme and print the new theme ID.
pub fn theme_duplicate(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let response = client.fetch_theme(theme_id)?;
    let theme = response.get("theme").unwrap_or(&response);

    let original_name = theme
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let new_name = format!("Copy of {}", original_name);

    let mut push_data = build_push_payload(theme);
    push_data["name"] = Value::String(new_name);
    // Never copy the default status to the duplicate
    push_data["default"] = Value::Bool(false);

    let new_id = client.create_theme(&push_data)?;
    println!("{}", new_id);
    Ok(())
}

/// Build a payload suitable for creating or updating a theme.
/// Strips server-generated and read-only fields.
fn build_push_payload(theme: &Value) -> Value {
    let mut map = serde_json::Map::new();
    for key in &[
        "name",
        "enabled",
        "user_selectable",
        "color_scheme_id",
        "theme_fields",
        "component",
    ] {
        if let Some(val) = theme.get(key) {
            map.insert(key.to_string(), val.clone());
        }
    }
    Value::Object(map)
}

fn ssh_target(discourse: &DiscourseConfig) -> String {
    discourse
        .ssh_host
        .clone()
        .unwrap_or_else(|| discourse.name.clone())
}

fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in replacements {
        out = out.replace(&format!("{{{}}}", key), value);
    }
    out
}

// ---------------------------------------------------------------------------
// Phase 1: component settings + enable/disable + attach/detach
// (spec/theme-management.md). Themes are handled as raw JSON values, matching
// the rest of this module.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ThemeSettingEntry {
    setting: String,
    #[serde(rename = "type")]
    kind: String,
    value: Value,
    default: Value,
}

/// Unwrap the `{ "theme": { … } }` envelope returned by some endpoints,
/// falling back to the bare object.
fn extract_theme(value: &Value) -> &Value {
    value.get("theme").unwrap_or(value)
}

/// Render a setting value for human-readable (text) output: strings bare,
/// null as empty, everything else as compact JSON.
fn value_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn theme_setting_entries(theme: &Value) -> Vec<ThemeSettingEntry> {
    theme
        .get("settings")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|s| ThemeSettingEntry {
                    setting: s
                        .get("setting")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    kind: s
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    value: s.get("value").cloned().unwrap_or(Value::Null),
                    default: s.get("default").cloned().unwrap_or(Value::Null),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// List a theme/component's settings (distinct from site settings).
pub fn theme_setting_list(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let response = client.fetch_theme(theme_id)?;
    let theme = extract_theme(&response);
    let entries = theme_setting_entries(theme);
    match format {
        ListFormat::Text => {
            if entries.is_empty() {
                println!("No settings found for theme {}.", theme_id);
                return Ok(());
            }
            for entry in &entries {
                println!("{} = {}", entry.setting, value_display(&entry.value));
            }
        }
        ListFormat::Json => println!("{}", serde_json::to_string_pretty(&entries)?),
        ListFormat::Yaml => println!("{}", serde_yaml::to_string(&entries)?),
    }
    Ok(())
}

/// Print a single theme/component setting's current value.
pub fn theme_setting_get(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
    key: &str,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let response = client.fetch_theme(theme_id)?;
    let theme = extract_theme(&response);
    let setting = theme
        .get("settings")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|s| s.get("setting").and_then(|v| v.as_str()) == Some(key))
        })
        .ok_or_else(|| not_found("theme setting", key))?;
    let value = setting.get("value").cloned().unwrap_or(Value::Null);
    println!("{}", value_display(&value));
    Ok(())
}

/// Set a single theme/component setting. The value is sent verbatim, so a
/// JSON-schema list setting takes its JSON text directly.
pub fn theme_setting_set(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
    key: &str,
    value: &str,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    if dry_run {
        println!(
            "[dry-run] {}: would set theme {} setting {} = {}",
            discourse.name, theme_id, key, value
        );
        return Ok(());
    }
    client.set_theme_setting(theme_id, key, value)?;
    println!("{}: set theme {} setting {}", discourse.name, theme_id, key);
    Ok(())
}

/// Enable or disable a theme/component (`PUT /admin/themes/:id.json` toggling
/// the `enabled` boolean).
pub fn theme_set_enabled(
    config: &Config,
    discourse_name: &str,
    theme_id: u64,
    enabled: bool,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let action = if enabled { "enable" } else { "disable" };
    if dry_run {
        println!("[dry-run] {}: would {} theme {}", discourse.name, action, theme_id);
        return Ok(());
    }
    client.update_theme(theme_id, &json!({ "enabled": enabled }))?;
    println!("{}: {}d theme {}", discourse.name, action, theme_id);
    Ok(())
}

/// Attach or detach a component to/from a parent theme. Reads the parent's
/// current `child_themes`, adds/removes the component id, and PUTs the full
/// replacement `child_theme_ids` set (disabled components stay in the list).
pub fn theme_set_child(
    config: &Config,
    discourse_name: &str,
    parent_id: u64,
    component_id: u64,
    attach: bool,
    dry_run: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let response = client.fetch_theme(parent_id)?;
    let theme = extract_theme(&response);
    let mut child_ids: Vec<u64> = theme
        .get("child_themes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.get("id").and_then(|v| v.as_u64()))
                .collect()
        })
        .unwrap_or_default();

    let present = child_ids.contains(&component_id);
    if attach && present {
        println!(
            "{}: component {} already attached to theme {}",
            discourse.name, component_id, parent_id
        );
        return Ok(());
    }
    if !attach && !present {
        println!(
            "{}: component {} is not attached to theme {}",
            discourse.name, component_id, parent_id
        );
        return Ok(());
    }
    if attach {
        child_ids.push(component_id);
    } else {
        child_ids.retain(|&id| id != component_id);
    }

    let (verb, prep) = if attach { ("attach", "to") } else { ("detach", "from") };
    if dry_run {
        println!(
            "[dry-run] {}: would {} component {} {} theme {} (child_theme_ids -> {:?})",
            discourse.name, verb, component_id, prep, parent_id, child_ids
        );
        return Ok(());
    }
    client.update_theme(parent_id, &json!({ "child_theme_ids": child_ids }))?;
    println!(
        "{}: {}ed component {} {} theme {}",
        discourse.name, verb, component_id, prep, parent_id
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_theme_unwraps_envelope_and_passes_bare() {
        let wrapped = json!({ "theme": { "id": 11, "name": "kitchen" } });
        assert_eq!(extract_theme(&wrapped).get("id").and_then(|v| v.as_u64()), Some(11));
        let bare = json!({ "id": 7, "name": "bare" });
        assert_eq!(extract_theme(&bare).get("id").and_then(|v| v.as_u64()), Some(7));
    }

    #[test]
    fn value_display_renders_each_json_kind() {
        assert_eq!(value_display(&json!("right")), "right");
        assert_eq!(value_display(&Value::Null), "");
        assert_eq!(value_display(&json!(true)), "true");
        assert_eq!(value_display(&json!(42)), "42");
        // json-schema list settings arrive as a JSON string already; an actual
        // array still renders as compact JSON for text output.
        assert_eq!(value_display(&json!(["a", "b"])), "[\"a\",\"b\"]");
    }

    #[test]
    fn theme_setting_entries_parses_settings_array() {
        let theme = json!({
            "settings": [
                { "setting": "links_position", "type": "enum", "default": "right", "value": "left" },
                { "setting": "header_links", "type": "string", "default": "[]", "value": "[{\"id\":1}]" }
            ]
        });
        let entries = theme_setting_entries(&theme);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].setting, "links_position");
        assert_eq!(entries[0].kind, "enum");
        assert_eq!(value_display(&entries[0].value), "left");
        assert_eq!(entries[1].setting, "header_links");
        assert_eq!(value_display(&entries[1].value), "[{\"id\":1}]");
    }

    #[test]
    fn theme_setting_entries_empty_when_absent() {
        assert!(theme_setting_entries(&json!({ "name": "no settings" })).is_empty());
    }
}
