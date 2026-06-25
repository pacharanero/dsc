use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::config::{Config, DiscourseConfig};
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::io::Write;
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

    let text = matches!(format, ListFormat::Text);
    // Column width known up front (from the config) so streamed rows align.
    let name_width = config
        .discourse
        .iter()
        .map(|d| d.name.len())
        .max()
        .unwrap_or(0)
        .max(4);

    // Signpost on stderr: this contacts every forum over the network (and SSH),
    // so it can take a while. stderr keeps json/yaml stdout clean.
    eprintln!(
        "Checking {} discourse(s) via API{}... contacts each over the network and can take a while.",
        config.discourse.len(),
        if skip_ssh { "" } else { " + SSH" }
    );

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
        let report = CheckReport {
            name: discourse.name.clone(),
            baseurl: discourse.baseurl.clone(),
            api,
            ssh,
        };
        // Text mode: stream each result the moment it lands, rather than
        // buffering the whole table to the end of a 30s run.
        if text {
            print_report_text(&report, name_width);
            let _ = std::io::stdout().flush();
        }
        reports.push(report);
    }

    let failed = reports
        .iter()
        .filter(|r| !(r.api.ok && r.ssh.as_ref().map(|s| s.ok).unwrap_or(true)))
        .count();

    match format {
        ListFormat::Text => eprintln!(
            "Done: {} ok, {} failed (of {}).",
            reports.len() - failed,
            failed,
            reports.len()
        ),
        ListFormat::Json => println!("{}", serde_json::to_string_pretty(&reports)?),
        ListFormat::Yaml => println!("{}", serde_yaml::to_string(&reports)?),
    }

    if failed == 0 {
        Ok(())
    } else {
        Err(anyhow!("{failed} discourse(s) failed checks"))
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

fn print_report_text(r: &CheckReport, name_width: usize) {
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
