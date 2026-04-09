use crate::commands::common::{open_url, select_discourse};
use crate::config::Config;
use anyhow::{Context, Result};

pub fn open_discourse(config: &Config, discourse_name: &str) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    open_url(&discourse.baseurl)
        .with_context(|| format!("opening browser for '{}'", discourse.baseurl))
}
