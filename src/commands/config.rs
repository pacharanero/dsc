use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::config::{Config, DiscourseConfig};
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct CheckReport {
    name: String,
    baseurl: String,
    api: CheckStatus,
    ssh: Option<CheckStatus>,
}

#[derive(Serialize)]
struct CheckStatus {
    ok: bool,
    detail: String,
}

pub fn config_check(config: &Config, format: ListFormat, skip_ssh: bool) -> Result<()> {
    if config.discourse.is_empty() {
        return Err(anyhow!("no discourses configured"));
    }

    let mut reports: Vec<CheckReport> = Vec::with_capacity(config.discourse.len());
    for discourse in &config.discourse {
        let api = check_api(discourse);
        let ssh = if skip_ssh {
            None
        } else {
            discourse
                .ssh_host
                .as_deref()
                .filter(|h| !h.trim().is_empty())
                .map(check_ssh)
        };
        reports.push(CheckReport {
            name: discourse.name.clone(),
            baseurl: discourse.baseurl.clone(),
            api,
            ssh,
        });
    }

    let all_ok = reports
        .iter()
        .all(|r| r.api.ok && r.ssh.as_ref().map(|s| s.ok).unwrap_or(true));

    match format {
        ListFormat::Text => print_text(&reports),
        ListFormat::Json => println!("{}", serde_json::to_string_pretty(&reports)?),
        ListFormat::Yaml => println!("{}", serde_yaml::to_string(&reports)?),
    }

    if all_ok {
        Ok(())
    } else {
        Err(anyhow!("one or more discourses failed checks"))
    }
}

fn check_api(discourse: &DiscourseConfig) -> CheckStatus {
    if discourse.baseurl.trim().is_empty() {
        return CheckStatus {
            ok: false,
            detail: "missing baseurl".to_string(),
        };
    }
    let client = match DiscourseClient::new(discourse) {
        Ok(client) => client,
        Err(err) => {
            return CheckStatus {
                ok: false,
                detail: format!("client init failed: {err}"),
            };
        }
    };
    match client.get("/about.json") {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                CheckStatus {
                    ok: true,
                    detail: format!("{} OK", status.as_u16()),
                }
            } else if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                CheckStatus {
                    ok: false,
                    detail: format!("{} — check apikey/api_username", status.as_u16()),
                }
            } else {
                CheckStatus {
                    ok: false,
                    detail: format!("HTTP {}", status.as_u16()),
                }
            }
        }
        Err(err) => CheckStatus {
            ok: false,
            detail: format!("request failed: {err}"),
        },
    }
}

fn check_ssh(host: &str) -> CheckStatus {
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "-o",
            "StrictHostKeyChecking=accept-new",
            host,
            "true",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(out) if out.status.success() => CheckStatus {
            ok: true,
            detail: format!("ssh {} OK", host),
        },
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first = stderr.lines().next().unwrap_or("").trim();
            CheckStatus {
                ok: false,
                detail: if first.is_empty() {
                    format!("ssh failed (exit {})", out.status)
                } else {
                    format!("ssh: {}", first)
                },
            }
        }
        Err(err) => CheckStatus {
            ok: false,
            detail: format!("ssh spawn failed: {err}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::check_api;
    use crate::config::DiscourseConfig;

    #[test]
    fn check_api_flags_empty_baseurl() {
        let discourse = DiscourseConfig {
            name: "empty".to_string(),
            baseurl: String::new(),
            ..DiscourseConfig::default()
        };
        let status = check_api(&discourse);
        assert!(!status.ok);
        assert!(
            status.detail.contains("missing baseurl"),
            "expected missing-baseurl detail, got {:?}",
            status.detail
        );
    }

    #[test]
    fn check_api_flags_whitespace_baseurl() {
        let discourse = DiscourseConfig {
            name: "ws".to_string(),
            baseurl: "   ".to_string(),
            ..DiscourseConfig::default()
        };
        let status = check_api(&discourse);
        assert!(!status.ok);
        assert!(status.detail.contains("missing baseurl"));
    }
}

fn print_text(reports: &[CheckReport]) {
    let name_width = reports.iter().map(|r| r.name.len()).max().unwrap_or(0).max(4);
    for r in reports {
        let api_mark = if r.api.ok { "ok " } else { "FAIL" };
        println!(
            "{:<width$}  api  {:<4}  {}",
            r.name,
            api_mark,
            r.api.detail,
            width = name_width
        );
        if let Some(ssh) = &r.ssh {
            let mark = if ssh.ok { "ok " } else { "FAIL" };
            println!(
                "{:<width$}  ssh  {:<4}  {}",
                r.name,
                mark,
                ssh.detail,
                width = name_width
            );
        }
    }
}

