use super::client::DiscourseClient;
use super::error::http_error;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagInfo {
    pub id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub count: u64,
    #[serde(default)]
    pub pm_count: u64,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    tags: Vec<TagInfo>,
}

impl DiscourseClient {
    /// List every tag visible to the authenticated user.
    pub fn list_tags(&self) -> Result<Vec<TagInfo>> {
        let response = self.get("/tags.json")?;
        let status = response.status();
        let text = response.text().context("reading tags response body")?;
        if !status.is_success() {
            return Err(http_error("tags request", status, &text));
        }
        let body: TagsResponse =
            serde_json::from_str(&text).context("parsing tags response json")?;
        Ok(body.tags)
    }

    /// Fetch the current tag list for a topic.
    pub fn fetch_topic_tags(&self, topic_id: u64) -> Result<Vec<String>> {
        let path = format!("/t/{}.json", topic_id);
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading topic response body")?;
        if !status.is_success() {
            return Err(http_error("topic request", status, &text));
        }
        let value: Value = serde_json::from_str(&text).context("parsing topic response json")?;
        let tags = value
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(tags)
    }

    /// Replace the full tag list on a topic. Returns the resulting tag set.
    pub fn set_topic_tags(&self, topic_id: u64, tags: &[String]) -> Result<Vec<String>> {
        let path = format!("/t/{}.json", topic_id);
        let payload: Vec<(&str, &str)> = if tags.is_empty() {
            // Discourse needs at least one form field to clear tags;
            // sending an empty `tags[]` removes them all.
            vec![("tags[]", "")]
        } else {
            tags.iter().map(|t| ("tags[]", t.as_str())).collect()
        };
        let response = self.send_retrying(|| Ok(self.put(&path)?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading set-tags response body")?;
        if !status.is_success() {
            return Err(http_error("set tags request", status, &text));
        }
        // Confirm the post-update state by re-reading the topic; the PUT response
        // shape is awkward to depend on across versions.
        self.fetch_topic_tags(topic_id)
    }
}
