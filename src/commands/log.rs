//! `dsc log staff` — read-only access to the staff action log (the admin
//! audit trail behind `/admin/logs/staff-action-logs` in the web UI).

use crate::api::{DiscourseClient, StaffActionLogFilter};
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, select_discourse};
use crate::config::Config;
use crate::utils::parse_since_cutoff;
use anyhow::Result;

#[allow(clippy::too_many_arguments)]
pub fn log_staff(
    config: &Config,
    discourse_name: &str,
    action: Option<&str>,
    acting_user: Option<&str>,
    target_user: Option<&str>,
    subject: Option<&str>,
    since: Option<&str>,
    limit: u32,
    format: ListFormat,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;

    let start_date = since_to_start_date(since)?;

    let filter = StaffActionLogFilter {
        action_name: action,
        acting_user,
        target_user,
        subject,
        start_date: start_date.as_deref(),
        limit,
    };
    let entries = client.fetch_staff_action_logs(&filter)?;

    match format {
        ListFormat::Text => {
            if entries.is_empty() {
                println!("No staff action log entries found.");
                return Ok(());
            }
            for e in &entries {
                let actor = e
                    .acting_user
                    .as_ref()
                    .map(|u| u.username.as_str())
                    .unwrap_or("-");
                let target = e
                    .target_user
                    .as_ref()
                    .map(|u| u.username.as_str())
                    .unwrap_or("-");
                let subject = e.subject.as_deref().unwrap_or("");
                println!(
                    "{}  {:<28}  actor={:<16}  target={:<16}  {}",
                    e.created_at, e.action_name, actor, target, subject
                );
            }
        }
        ListFormat::Json => println!("{}", serde_json::to_string_pretty(&entries)?),
        ListFormat::Yaml => println!("{}", serde_yaml::to_string(&entries)?),
    }
    Ok(())
}

/// Convert a `--since` value (relative duration or ISO-8601) into the
/// `YYYY-MM-DD` shape Discourse's `start_date` filter expects.
fn since_to_start_date(since: Option<&str>) -> Result<Option<String>> {
    since
        .map(parse_since_cutoff)
        .transpose()
        .map(|cutoff| cutoff.map(|c| c.format("%Y-%m-%d").to_string()))
}

#[cfg(test)]
mod tests {
    use super::since_to_start_date;

    #[test]
    fn none_since_yields_none_start_date() {
        assert_eq!(since_to_start_date(None).unwrap(), None);
    }

    #[test]
    fn relative_duration_yields_a_date() {
        let got = since_to_start_date(Some("7d")).unwrap();
        assert!(got.is_some());
        let date = got.unwrap();
        assert_eq!(date.len(), 10, "expected YYYY-MM-DD, got {date:?}");
    }

    #[test]
    fn iso_date_passes_through() {
        let got = since_to_start_date(Some("2026-04-01")).unwrap();
        assert_eq!(got, Some("2026-04-01".to_string()));
    }

    #[test]
    fn invalid_since_is_an_error() {
        assert!(since_to_start_date(Some("not-a-date")).is_err());
    }
}
