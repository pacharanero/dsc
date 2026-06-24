use crate::api::DiscourseClient;
use crate::commands::common::{ensure_api_credentials, select_discourse};
use crate::config::Config;
use anyhow::Result;

/// Print a configured forum's live Discourse version and git commit, read from
/// `/about.json`. Uses the configured API key, so it works even on
/// login-required forums where an anonymous request would be rejected.
pub fn forum_version(config: &Config, discourse_name: &str) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let info = client.fetch_version_info()?;
    let version = info.version.as_deref().unwrap_or("(unknown)");
    let commit = info.commit.as_deref().unwrap_or("(unknown)");
    println!("{}: Discourse {} ({})", discourse.name, version, commit);
    Ok(())
}
