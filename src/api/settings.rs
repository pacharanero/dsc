use super::client::DiscourseClient;
use super::error::http_error;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;

impl DiscourseClient {
    /// Update a site setting by name (admin only).
    pub fn update_site_setting(&self, setting: &str, value: &str) -> Result<()> {
        let setting = setting.trim();
        if setting.is_empty() {
            return Err(anyhow!("missing site setting name for site setting update"));
        }
        if setting.chars().any(|ch| ch.is_whitespace() || ch == '/') {
            return Err(anyhow!(
                "site setting name contains invalid characters: {}",
                setting
            ));
        }
        let path = format!("/admin/site_settings/{}.json", setting);
        let payload = [("value", value)];
        let response = self.send_retrying(|| Ok(self.put(&path)?.form(&payload)))?;
        let status = response.status();
        let text = response
            .text()
            .context("reading site setting update response")?;
        if !status.is_success() {
            return Err(http_error("update site setting request", status, &text));
        }
        Ok(())
    }

    /// Fetch all site settings (admin only). Returns raw JSON value.
    pub fn list_site_settings(&self) -> Result<Value> {
        let response = self.get("/admin/site_settings.json")?;
        let status = response.status();
        let text = response
            .text()
            .context("reading site settings list response")?;
        if !status.is_success() {
            return Err(http_error("list site settings request", status, &text));
        }
        let value: Value =
            serde_json::from_str(&text).context("parsing site settings list response")?;
        Ok(value)
    }

    /// Fetch a single site setting by name (admin only).
    /// Returns the value as a string, or an error if not found.
    pub fn fetch_site_setting(&self, setting: &str) -> Result<String> {
        let setting = setting.trim();
        if setting.is_empty() {
            return Err(anyhow!("missing site setting name"));
        }
        // The admin site settings API returns all settings; we filter by name.
        let all = self.list_site_settings()?;
        // Response shape: { "site_settings": [ { "setting": "...", "value": ... }, ... ] }
        let settings = all
            .get("site_settings")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for entry in &settings {
            let name = entry.get("setting").and_then(|v| v.as_str()).unwrap_or("");
            if name == setting {
                let value = entry.get("value").cloned().unwrap_or(Value::Null);
                let display = match &value {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                };
                return Ok(display);
            }
        }
        Err(anyhow!("setting not found: {}", setting))
    }
}
