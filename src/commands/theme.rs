use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::commands::common::{emit_result, ensure_api_credentials, not_found, select_discourse};
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
    format: ListFormat,
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
    emit_result(format, &json!({ "id": new_id }), &new_id.to_string())
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
    format: ListFormat,
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
    emit_result(
        format,
        &json!({ "setting": key, "value": value }),
        &value_display(&value),
    )
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

#[derive(Debug, Serialize)]
struct ThemeRelation {
    id: u64,
    name: String,
}

#[derive(Debug, Serialize)]
struct ThemeShow {
    id: u64,
    name: String,
    component: bool,
    enabled: bool,
    default: bool,
    user_selectable: bool,
    color_scheme_id: Option<u64>,
    parent_themes: Vec<ThemeRelation>,
    child_themes: Vec<ThemeRelation>,
    settings_count: usize,
    fields: Vec<String>,
}

/// Parse an array of `{id, name}` theme relations (child/parent themes),
/// skipping entries missing an id.
fn theme_relations(theme: &Value, key: &str) -> Vec<ThemeRelation> {
    theme
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|r| {
                    let id = r.get("id").and_then(|v| v.as_u64())?;
                    let name = r
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    Some(ThemeRelation { id, name })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Inventory of editable `theme_fields` as `target/name` strings (e.g.
/// `common/scss`). Parsed defensively so an unexpected entry shape just
/// contributes nothing rather than erroring.
fn theme_field_inventory(theme: &Value) -> Vec<String> {
    theme
        .get("theme_fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let name = f.get("name").and_then(|v| v.as_str())?;
                    let target = f.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    if target.is_empty() {
                        Some(name.to_string())
                    } else {
                        Some(format!("{}/{}", target, name))
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn build_theme_show(theme: &Value, theme_id: u64) -> ThemeShow {
    ThemeShow {
        id: theme.get("id").and_then(|v| v.as_u64()).unwrap_or(theme_id),
        name: theme
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        component: theme.get("component").and_then(|v| v.as_bool()).unwrap_or(false),
        enabled: theme.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false),
        default: theme.get("default").and_then(|v| v.as_bool()).unwrap_or(false),
        user_selectable: theme
            .get("user_selectable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        color_scheme_id: theme.get("color_scheme_id").and_then(|v| v.as_u64()),
        parent_themes: theme_relations(theme, "parent_themes"),
        child_themes: theme_relations(theme, "child_themes"),
        settings_count: theme_setting_entries(theme).len(),
        fields: theme_field_inventory(theme),
    }
}

fn format_relations(rels: &[ThemeRelation]) -> String {
    if rels.is_empty() {
        "(none)".to_string()
    } else {
        rels.iter()
            .map(|r| format!("{} - {}", r.id, r.name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Show a richer view of one theme/component than `theme list`: type, enabled
/// and default flags, parents, attached children, settings count, and the
/// editable field inventory.
pub fn theme_show(
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
    let show = build_theme_show(theme, theme_id);
    match format {
        ListFormat::Json => println!("{}", serde_json::to_string_pretty(&show)?),
        ListFormat::Yaml => println!("{}", serde_yaml::to_string(&show)?),
        ListFormat::Text => {
            println!("{} - {}", show.id, show.name);
            println!(
                "  type:            {}",
                if show.component { "component" } else { "theme" }
            );
            println!("  enabled:         {}", show.enabled);
            println!("  default:         {}", show.default);
            println!("  user-selectable: {}", show.user_selectable);
            if let Some(cs) = show.color_scheme_id {
                println!("  color scheme:    {}", cs);
            }
            println!("  parents:         {}", format_relations(&show.parent_themes));
            println!("  children:        {}", format_relations(&show.child_themes));
            println!("  settings:        {}", show.settings_count);
            let fields = if show.fields.is_empty() {
                "(none)".to_string()
            } else {
                show.fields.join(", ")
            };
            println!("  fields:          {}", fields);
        }
    }
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

    #[test]
    fn theme_relations_parses_id_name_pairs() {
        let theme = json!({
            "child_themes": [
                { "id": 8, "name": "Header Submenus" },
                { "id": 14, "name": "Dropdown Header" },
                { "name": "no id, skipped" }
            ]
        });
        let rels = theme_relations(&theme, "child_themes");
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0].id, 8);
        assert_eq!(rels[1].name, "Dropdown Header");
        assert!(theme_relations(&theme, "parent_themes").is_empty());
    }

    #[test]
    fn theme_field_inventory_joins_target_and_name() {
        let theme = json!({
            "theme_fields": [
                { "target": "common", "name": "scss", "value": "body{}" },
                { "target": "desktop", "name": "scss", "value": "" },
                { "target": "", "name": "extra_js", "value": "" },
                { "value": "no name, skipped" }
            ]
        });
        let fields = theme_field_inventory(&theme);
        assert_eq!(fields, vec!["common/scss", "desktop/scss", "extra_js"]);
    }

    #[test]
    fn build_theme_show_summarises_core_fields() {
        let theme = json!({
            "id": 11,
            "name": "kitchen-customisations",
            "component": false,
            "enabled": true,
            "default": false,
            "user_selectable": true,
            "child_themes": [{ "id": 14, "name": "Dropdown Header" }],
            "settings": [{ "setting": "links_position", "value": "left" }],
            "theme_fields": [{ "target": "common", "name": "scss", "value": "x" }]
        });
        let show = build_theme_show(&theme, 11);
        assert_eq!(show.id, 11);
        assert!(!show.component);
        assert!(show.enabled);
        assert_eq!(show.child_themes.len(), 1);
        assert_eq!(show.settings_count, 1);
        assert_eq!(show.fields, vec!["common/scss"]);
    }
}
