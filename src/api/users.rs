use super::client::DiscourseClient;
use super::error::http_error;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One row from /admin/users/list/<type>.json.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserSummary {
    pub id: u64,
    pub username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub trust_level: Option<u64>,
    #[serde(default)]
    pub admin: Option<bool>,
    #[serde(default)]
    pub moderator: Option<bool>,
    #[serde(default)]
    pub suspended: Option<bool>,
    #[serde(default)]
    pub silenced: Option<bool>,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Distilled /users/<username>.json payload.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserDetail {
    pub id: u64,
    pub username: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub trust_level: Option<u64>,
    #[serde(default)]
    pub admin: Option<bool>,
    #[serde(default)]
    pub moderator: Option<bool>,
    #[serde(default)]
    pub suspended_till: Option<String>,
    #[serde(default)]
    pub silenced_till: Option<String>,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub post_count: Option<u64>,
    #[serde(default)]
    pub groups: Vec<Value>,
}

impl DiscourseClient {
    /// List users via the admin users endpoint.
    ///
    /// `listing` is one of: `active` (default), `new`, `staff`, `suspended`,
    /// `silenced`, `staged`. Discourse paginates 100 per page.
    pub fn admin_list_users(&self, listing: &str, page: u32) -> Result<Vec<UserSummary>> {
        let path = format!(
            "/admin/users/list/{}.json?show_emails=true&page={}",
            listing, page
        );
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading user list response")?;
        if !status.is_success() {
            return Err(http_error("admin user list request", status, &text));
        }
        let users: Vec<UserSummary> =
            serde_json::from_str(&text).context("parsing user list response")?;
        Ok(users)
    }

    /// Look up a user by username (public endpoint).
    pub fn fetch_user_detail(&self, username: &str) -> Result<UserDetail> {
        let path = format!("/u/{}.json", username);
        let response = self.get(&path)?;
        let status = response.status();
        let text = response.text().context("reading user detail response")?;
        if !status.is_success() {
            return Err(http_error("user detail request", status, &text));
        }
        let value: Value =
            serde_json::from_str(&text).context("parsing user detail response")?;
        let user = value
            .get("user")
            .ok_or_else(|| anyhow!("user detail response missing `user` field"))?;
        let detail: UserDetail =
            serde_json::from_value(user.clone()).context("deserialising user detail")?;
        Ok(detail)
    }

    /// Suspend a user by ID. `until` is an ISO-8601 timestamp (or any string
    /// Discourse accepts, like "forever"); `reason` is mandatory from the UI
    /// but Discourse accepts empty via the API.
    pub fn suspend_user(&self, user_id: u64, until: &str, reason: &str) -> Result<()> {
        let payload = [("suspend_until", until), ("reason", reason)];
        self.put_admin_user_action(user_id, "suspend", &payload, "suspend user request")
    }

    /// Unsuspend a user by ID.
    pub fn unsuspend_user(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(user_id, "unsuspend", &[], "unsuspend user request")
    }

    /// Silence a user by ID. Optional `silenced_till` (Discourse-accepted
    /// timestamp string) and `reason`; both default to empty.
    pub fn silence_user(&self, user_id: u64, until: &str, reason: &str) -> Result<()> {
        let mut payload: Vec<(&str, &str)> = Vec::new();
        if !until.is_empty() {
            payload.push(("silenced_till", until));
        }
        if !reason.is_empty() {
            payload.push(("reason", reason));
        }
        self.put_admin_user_action(user_id, "silence", &payload, "silence user request")
    }

    /// Unsilence a user by ID.
    pub fn unsilence_user(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(user_id, "unsilence", &[], "unsilence user request")
    }

    /// Grant admin to a user.
    pub fn grant_admin(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(user_id, "grant_admin", &[], "grant admin request")
    }

    /// Revoke admin from a user.
    pub fn revoke_admin(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(user_id, "revoke_admin", &[], "revoke admin request")
    }

    /// Grant moderator to a user.
    pub fn grant_moderation(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(
            user_id,
            "grant_moderation",
            &[],
            "grant moderation request",
        )
    }

    /// Revoke moderator from a user.
    pub fn revoke_moderation(&self, user_id: u64) -> Result<()> {
        self.put_admin_user_action(
            user_id,
            "revoke_moderation",
            &[],
            "revoke moderation request",
        )
    }

    /// Create a user. `password` is optional — omit to require the new user
    /// to reset it via the email flow. `active=true` and `approved=true` are
    /// passed so admin-created accounts skip the activation and approval
    /// dances. Returns the new user id on success.
    pub fn create_user(
        &self,
        email: &str,
        username: &str,
        password: Option<&str>,
        name: Option<&str>,
        approve: bool,
    ) -> Result<u64> {
        let mut payload: Vec<(&str, &str)> = vec![
            ("email", email),
            ("username", username),
            ("active", "true"),
        ];
        if approve {
            payload.push(("approved", "true"));
        }
        if let Some(p) = password {
            payload.push(("password", p));
        }
        if let Some(n) = name {
            if !n.is_empty() {
                payload.push(("name", n));
            }
        }
        let response = self.send_retrying(|| Ok(self.post("/u.json")?.form(&payload)))?;
        let status = response.status();
        let text = response.text().context("reading user create response")?;
        if !status.is_success() {
            return Err(http_error("user create request", status, &text));
        }
        let value: Value =
            serde_json::from_str(&text).context("parsing user create response")?;
        // Discourse wraps this variably depending on version; grab user_id from
        // the top level first, then fall back to `user.id`.
        let id = value
            .get("user_id")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                value
                    .get("user")
                    .and_then(|u| u.get("id"))
                    .and_then(|v| v.as_u64())
            })
            .ok_or_else(|| anyhow!("user create response missing user id: {}", text))?;
        Ok(id)
    }

    /// Trigger the "forgot password" email flow for a user. Accepts username
    /// or email as `login`. Discourse returns a generic success message
    /// regardless of whether the user exists (to prevent enumeration).
    pub fn trigger_password_reset(&self, login: &str) -> Result<()> {
        let payload = [("login", login)];
        let response = self
            .send_retrying(|| Ok(self.post("/session/forgot_password.json")?.form(&payload)))?;
        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(http_error("password reset request", status, &text));
        }
        Ok(())
    }

    /// Admin-set a user's primary email address.
    pub fn set_user_email(&self, username: &str, email: &str) -> Result<()> {
        let path = format!("/u/{}/preferences/email.json", username);
        let payload = [("email", email)];
        let response = self.send_retrying(|| Ok(self.post(&path)?.form(&payload)))?;
        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(http_error("email set request", status, &text));
        }
        Ok(())
    }

    fn put_admin_user_action(
        &self,
        user_id: u64,
        action: &str,
        payload: &[(&str, &str)],
        action_label: &str,
    ) -> Result<()> {
        let path = format!("/admin/users/{}/{}.json", user_id, action);
        let response = self.send_retrying(|| {
            let rb = self.put(&path)?;
            Ok(if payload.is_empty() {
                rb
            } else {
                rb.form(payload)
            })
        })?;
        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .unwrap_or_else(|_| "<failed to read response body>".to_string());
            return Err(http_error(action_label, status, &text));
        }
        Ok(())
    }
}
