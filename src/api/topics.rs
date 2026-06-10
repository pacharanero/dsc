use super::client::DiscourseClient;
use super::error::http_error;
use super::models::{CreatePostResponse, TopicResponse};
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostInfo {
    pub id: u64,
    pub topic_id: u64,
    #[serde(default)]
    pub post_number: Option<u64>,
    #[serde(default)]
    pub raw: Option<String>,
}

/// Distilled row from /topics/private-messages-*.json.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PmTopicSummary {
    pub id: u64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub posts_count: Option<u64>,
    #[serde(default)]
    pub last_posted_at: Option<String>,
    #[serde(default)]
    pub last_poster_username: Option<String>,
    #[serde(default)]
    pub unread: Option<u64>,
}

impl DiscourseClient {
    /// Fetch a topic by ID.
    pub fn fetch_topic(&self, topic_id: u64, include_raw: bool) -> Result<TopicResponse> {
        let path = if include_raw {
            format!("/t/{}.json?include_raw=1", topic_id)
        } else {
            format!("/t/{}.json", topic_id)
        };
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading topic response body")?;
        if !status.is_success() {
            return Err(http_error("topic request", status, &text));
        }
        let body: TopicResponse = serde_json::from_str(&text).context("parsing topic json")?;
        Ok(body)
    }

    /// Fetch every post in a topic, in order.
    ///
    /// Discourse paginates `/t/{id}.json` at 20 posts per page. The first
    /// response also includes `post_stream.stream`, the flat array of every
    /// post ID in the thread. We page-1 first to learn the stream, then
    /// batch-fetch any remaining post IDs via
    /// `/t/{id}/posts.json?post_ids[]=…&include_raw=1`. Returns posts in
    /// stream order (matches topic display order).
    pub fn fetch_topic_all_posts(&self, topic_id: u64) -> Result<TopicResponse> {
        let mut topic = self.fetch_topic(topic_id, true)?;

        // Build the set of IDs we already have from page 1.
        let have: std::collections::HashSet<u64> =
            topic.post_stream.posts.iter().map(|p| p.id).collect();
        let missing: Vec<u64> = topic
            .post_stream
            .stream
            .iter()
            .copied()
            .filter(|id| !have.contains(id))
            .collect();

        // Batch-fetch missing posts in chunks of 20 (Discourse's page size).
        for chunk in missing.chunks(20) {
            let query: Vec<String> = chunk
                .iter()
                .map(|id| format!("post_ids[]={}", id))
                .collect();
            let path = format!(
                "/t/{}/posts.json?include_raw=1&{}",
                topic_id,
                query.join("&")
            );
            let response = self.get(&path)?;
            let status = response.status();
            let text = response.text().context("reading topic posts response body")?;
            if !status.is_success() {
                return Err(http_error("topic posts request", status, &text));
            }
            let body: TopicResponse = serde_json::from_str(&text)
                .context("parsing topic posts response")?;
            topic.post_stream.posts.extend(body.post_stream.posts);
        }

        // Reorder posts to match the canonical stream order.
        if !topic.post_stream.stream.is_empty() {
            let order: std::collections::HashMap<u64, usize> = topic
                .post_stream
                .stream
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, i))
                .collect();
            topic
                .post_stream
                .posts
                .sort_by_key(|p| order.get(&p.id).copied().unwrap_or(usize::MAX));
        }

        Ok(topic)
    }

    /// Fetch a post by ID and return its raw content.
    pub fn fetch_post_raw(&self, post_id: u64) -> Result<Option<String>> {
        Ok(self.fetch_post(post_id)?.raw)
    }

    /// Fetch a post's metadata (id, topic_id, post_number, raw).
    pub fn fetch_post(&self, post_id: u64) -> Result<PostInfo> {
        let path = format!("/posts/{}.json?include_raw=1", post_id);
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading post response body")?;
        if !status.is_success() {
            return Err(http_error("post request", status, &text));
        }
        let info: PostInfo = serde_json::from_str(&text).context("parsing post response")?;
        Ok(info)
    }

    /// Soft-delete a post by ID (DELETE /posts/:id.json).
    pub fn delete_post(&self, post_id: u64) -> Result<()> {
        let path = format!("/posts/{}.json", post_id);
        let response = self.send_retrying(|| Ok(self.delete_builder(&path)?))?;
        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(http_error("delete post request", status, &text));
        }
        Ok(())
    }

    /// Move one or more posts from their current topic to another topic.
    ///
    /// `source_topic_id` is the topic the posts currently live in.
    /// `post_ids` are the post IDs to move. `dest_topic_id` is where they land.
    /// Returns the new URL of the moved posts' topic.
    pub fn move_posts(
        &self,
        source_topic_id: u64,
        post_ids: &[u64],
        dest_topic_id: u64,
    ) -> Result<String> {
        if post_ids.is_empty() {
            return Err(anyhow!("no post IDs supplied to move"));
        }
        let dest = dest_topic_id.to_string();
        let path = format!("/t/{}/move-posts.json", source_topic_id);
        let mut payload: Vec<(String, String)> = Vec::new();
        payload.push(("destination_topic_id".to_string(), dest.clone()));
        for id in post_ids {
            payload.push(("post_ids[]".to_string(), id.to_string()));
        }
        let response = self.send_retrying(|| Ok(self.post(&path)?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading move-posts response")?;
        if !status.is_success() {
            return Err(http_error("move posts request", status, &text));
        }
        let value: Value =
            serde_json::from_str(&text).context("parsing move-posts response")?;
        let url = value
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("/t/{}", dest));
        Ok(url)
    }

    /// Update a post by ID.
    pub fn update_post(&self, post_id: u64, raw: &str) -> Result<()> {
        let path = format!("/posts/{}.json", post_id);
        let payload = [("post[raw]", raw)];
        let response = self.send_retrying(|| Ok(self.put(&path)?.form(&payload)))?;
        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(http_error("update post request", status, &text));
        }
        Ok(())
    }

    /// Create a new topic in a category.
    pub fn create_topic(&self, category_id: u64, title: &str, raw: &str) -> Result<u64> {
        let category = category_id.to_string();
        let payload = [("title", title), ("raw", raw), ("category", &category)];
        let response = self.send_retrying(|| Ok(self.post("/posts.json")?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading create response body")?;
        if !status.is_success() {
            return Err(http_error("create topic request", status, &text));
        }
        let body: CreatePostResponse =
            serde_json::from_str(&text).context("parsing create topic response")?;
        Ok(body.topic_id)
    }

    /// Send a private message. `recipients` is comma-joined into Discourse's
    /// `target_recipients` field (usernames or group names accepted).
    /// Returns the new topic_id of the PM thread.
    pub fn create_private_message(
        &self,
        recipients: &[String],
        title: &str,
        raw: &str,
    ) -> Result<u64> {
        let recipients_csv = recipients.join(",");
        let payload = [
            ("title", title),
            ("raw", raw),
            ("archetype", "private_message"),
            ("target_recipients", recipients_csv.as_str()),
        ];
        let response = self.send_retrying(|| Ok(self.post("/posts.json")?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading PM create response body")?;
        if !status.is_success() {
            return Err(http_error("create PM request", status, &text));
        }
        let body: CreatePostResponse =
            serde_json::from_str(&text).context("parsing PM create response")?;
        Ok(body.topic_id)
    }

    /// List private messages for the given user. `direction` is one of
    /// `inbox` (received), `sent`, `archive`, `unread`, `new`. Returns
    /// distilled topic summaries.
    pub fn list_private_messages(
        &self,
        username: &str,
        direction: &str,
    ) -> Result<Vec<PmTopicSummary>> {
        let path = match direction {
            "inbox" => format!("/topics/private-messages/{}.json", username),
            "sent" => format!("/topics/private-messages-sent/{}.json", username),
            "archive" => format!("/topics/private-messages-archive/{}.json", username),
            "unread" => format!("/topics/private-messages-unread/{}.json", username),
            "new" => format!("/topics/private-messages-new/{}.json", username),
            other => format!("/topics/private-messages-{}/{}.json", other, username),
        };
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading PM list response")?;
        if !status.is_success() {
            return Err(http_error("PM list request", status, &text));
        }
        let value: Value = serde_json::from_str(&text).context("parsing PM list response")?;
        let topics = value
            .get("topic_list")
            .and_then(|tl| tl.get("topics"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value::<PmTopicSummary>(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
        Ok(topics)
    }

    /// Create a reply post in a topic.
    pub fn create_post(&self, topic_id: u64, raw: &str) -> Result<u64> {
        let topic = topic_id.to_string();
        let payload = [("topic_id", topic.as_str()), ("raw", raw)];
        let response = self.send_retrying(|| Ok(self.post("/posts.json")?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading create response body")?;
        if !status.is_success() {
            return Err(http_error("create post request", status, &text));
        }
        let body: CreatePostResponse =
            serde_json::from_str(&text).context("parsing create post response")?;
        Ok(body.id)
    }
}
