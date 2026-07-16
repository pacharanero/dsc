use super::client::DiscourseClient;
use super::error::http_error;
use super::search::urlencode_form;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// The acting or target user embedded in a staff action log entry —
/// Discourse's `BasicUserSerializer`, distilled to what callers need.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffLogUser {
    pub id: i64,
    pub username: String,
}

/// One row from `/admin/logs/staff_action_logs.json` (a `UserHistory`
/// record). Field set matches Discourse's `UserHistorySerializer`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaffActionLog {
    pub id: u64,
    /// Symbolic action name, e.g. `change_site_setting`, `suspend_user`.
    pub action_name: String,
    #[serde(default)]
    pub acting_user: Option<StaffLogUser>,
    #[serde(default)]
    pub target_user: Option<StaffLogUser>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub previous_value: Option<String>,
    #[serde(default)]
    pub new_value: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct StaffActionLogsResponse {
    #[serde(default)]
    staff_action_logs: Vec<StaffActionLog>,
}

/// Query filters for `/admin/logs/staff_action_logs.json`, mirroring
/// Discourse's `UserHistory.staff_filters`.
#[derive(Debug, Default, Clone)]
pub struct StaffActionLogFilter<'a> {
    pub action_name: Option<&'a str>,
    pub acting_user: Option<&'a str>,
    pub target_user: Option<&'a str>,
    pub subject: Option<&'a str>,
    /// Only entries at or after this date (`YYYY-MM-DD`).
    pub start_date: Option<&'a str>,
    /// Rows to fetch; Discourse caps this server-side at 200.
    pub limit: u32,
}

impl DiscourseClient {
    /// Fetch a page of the staff action log (the admin audit trail).
    pub fn fetch_staff_action_logs(
        &self,
        filter: &StaffActionLogFilter,
    ) -> Result<Vec<StaffActionLog>> {
        let path = format!("/admin/logs/staff_action_logs.json?{}", build_query(filter));
        let response = self.get(&path)?;
        let status = response.status();
        let text = response
            .text()
            .context("reading staff action log response")?;
        if !status.is_success() {
            return Err(http_error("staff action log request", status, &text));
        }
        let body: StaffActionLogsResponse =
            serde_json::from_str(&text).context("parsing staff action log response")?;
        Ok(body.staff_action_logs)
    }
}

/// Build the `application/x-www-form-urlencoded` query string for
/// `/admin/logs/staff_action_logs.json` from a filter.
fn build_query(filter: &StaffActionLogFilter) -> String {
    let mut params = vec![format!("limit={}", filter.limit.min(200))];
    if let Some(v) = filter.action_name {
        params.push(format!("action_name={}", urlencode_form(v)));
    }
    if let Some(v) = filter.acting_user {
        params.push(format!("acting_user={}", urlencode_form(v)));
    }
    if let Some(v) = filter.target_user {
        params.push(format!("target_user={}", urlencode_form(v)));
    }
    if let Some(v) = filter.subject {
        params.push(format!("subject={}", urlencode_form(v)));
    }
    if let Some(v) = filter.start_date {
        params.push(format!("start_date={}", urlencode_form(v)));
    }
    params.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_only_sends_limit() {
        let filter = StaffActionLogFilter {
            limit: 50,
            ..Default::default()
        };
        assert_eq!(build_query(&filter), "limit=50");
    }

    #[test]
    fn limit_is_capped_at_200() {
        let filter = StaffActionLogFilter {
            limit: 5000,
            ..Default::default()
        };
        assert_eq!(build_query(&filter), "limit=200");
    }

    #[test]
    fn all_filters_are_included_and_encoded() {
        let filter = StaffActionLogFilter {
            action_name: Some("change_site_setting"),
            acting_user: Some("alice"),
            target_user: Some("bob smith"),
            subject: Some("login required"),
            start_date: Some("2026-07-01"),
            limit: 10,
        };
        assert_eq!(
            build_query(&filter),
            "limit=10&action_name=change_site_setting&acting_user=alice&target_user=bob+smith&subject=login+required&start_date=2026-07-01"
        );
    }

    #[test]
    fn deserializes_entry_with_null_users() {
        let raw = r#"{
            "staff_action_logs": [
                {
                    "id": 1,
                    "action_name": "change_site_setting",
                    "acting_user": {"id": 1, "username": "system"},
                    "target_user": null,
                    "subject": "title",
                    "details": null,
                    "previous_value": "Old",
                    "new_value": "New",
                    "created_at": "2026-07-01T00:00:00.000Z"
                }
            ]
        }"#;
        let body: StaffActionLogsResponse = serde_json::from_str(raw).expect("parse");
        assert_eq!(body.staff_action_logs.len(), 1);
        let entry = &body.staff_action_logs[0];
        assert_eq!(entry.action_name, "change_site_setting");
        assert_eq!(entry.acting_user.as_ref().unwrap().username, "system");
        assert!(entry.target_user.is_none());
    }
}
