use crate::api::DiscourseClient;
use crate::cli::ListFormat;
use crate::commands::common::{ensure_api_credentials, select_discourse};
use crate::config::Config;
use crate::utils::slugify;
use anyhow::{Context, Result, anyhow};
use base64::Engine;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn add_emoji(
    config: &Config,
    discourse_name: &str,
    emoji_path: &Path,
    emoji_name: Option<&str>,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let mut existing_names = existing_emoji_names(&client).unwrap_or_else(|err| {
        eprintln!(
            "Warning: failed to list existing emojis for idempotent checks: {}",
            err
        );
        HashSet::new()
    });

    if emoji_path.is_dir() {
        if emoji_name.is_some() {
            return Err(anyhow!(
                "emoji name is not allowed when uploading a directory"
            ));
        }
        let mut files = Vec::new();
        for entry in
            fs::read_dir(emoji_path).with_context(|| format!("reading {}", emoji_path.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if !is_emoji_file(&path) {
                continue;
            }
            files.push(path);
        }
        files.sort();
        if files.is_empty() {
            return Err(anyhow!("no emoji image files found in directory"));
        }

        let mut uploaded = 0usize;
        let mut skipped_existing = 0usize;
        let mut failures: Vec<(String, String, String)> = Vec::new();
        for path in files {
            let name = emoji_name_from_path(&path)?;
            let key = emoji_key(&name);
            if existing_names.contains(&key) {
                skipped_existing += 1;
                println!("Skipped existing emoji {} from {}", name, path.display());
                continue;
            }

            match client.upload_emoji(&path, &name) {
                Ok(_) => {
                    uploaded += 1;
                    existing_names.insert(key);
                    println!("Uploaded emoji {} from {}", name, path.display());
                }
                Err(err) if is_duplicate_emoji_error(&err) => {
                    skipped_existing += 1;
                    existing_names.insert(key);
                    println!("Skipped existing emoji {} from {}", name, path.display());
                }
                Err(err) => {
                    failures.push((name, path.display().to_string(), err.to_string()));
                }
            }
        }

        if !failures.is_empty() {
            eprintln!("Emoji upload failures:");
            for (name, path, reason) in &failures {
                eprintln!("- {} ({}) => {}", name, path, reason);
            }
        }
        println!(
            "Emoji bulk upload summary: uploaded={}, skipped_existing={}, failed={}",
            uploaded,
            skipped_existing,
            failures.len()
        );
        if !failures.is_empty() {
            return Err(anyhow!(
                "{} emoji uploads failed; see failure summary above",
                failures.len()
            ));
        }
        return Ok(());
    }

    let name = match emoji_name {
        Some(name) => name.to_string(),
        None => emoji_name_from_path(emoji_path)?,
    };
    let key = emoji_key(&name);
    if existing_names.contains(&key) {
        println!(
            "Skipped existing emoji {} from {}",
            name,
            emoji_path.display()
        );
        return Ok(());
    }

    match client.upload_emoji(emoji_path, &name) {
        Ok(_) => {
            println!("Uploaded emoji {} from {}", name, emoji_path.display());
            Ok(())
        }
        Err(err) if is_duplicate_emoji_error(&err) => {
            println!(
                "Skipped existing emoji {} from {}",
                name,
                emoji_path.display()
            );
            Ok(())
        }
        Err(err) => Err(err),
    }?;
    Ok(())
}

pub fn list_emojis(
    config: &Config,
    discourse_name: &str,
    format: ListFormat,
    verbose: bool,
    inline: bool,
) -> Result<()> {
    let discourse = select_discourse(config, Some(discourse_name))?;
    ensure_api_credentials(discourse)?;
    let client = DiscourseClient::new(discourse)?;
    let mut emojis = client.list_custom_emojis()?;
    emojis.sort_by(|a, b| a.name.cmp(&b.name));

    match format {
        ListFormat::Text => {
            if emojis.is_empty() && !verbose {
                println!("No emojis found.");
                return Ok(());
            }

            if inline {
                if let Some(protocol) = detect_inline_protocol() {
                    print_inline_emojis(&emojis, protocol)?;
                } else {
                    print_emojis_table(&emojis);
                }
            } else {
                print_emojis_table(&emojis);
            }
        }
        ListFormat::Json => {
            if inline {
                return Err(anyhow!("--inline is only supported with --format text"));
            }
            let raw = serde_json::to_string_pretty(&emojis)?;
            println!("{}", raw);
        }
        ListFormat::Yaml => {
            if inline {
                return Err(anyhow!("--inline is only supported with --format text"));
            }
            let raw = serde_yaml::to_string(&emojis)?;
            println!("{}", raw);
        }
    }
    Ok(())
}

fn print_emojis_table(emojis: &[crate::api::CustomEmoji]) {
    println!("name\turl");
    for emoji in emojis {
        println!("{}\t{}", emoji.name, emoji.url);
    }
}

#[derive(Clone, Copy)]
enum InlineProtocol {
    Iterm2,
    Kitty,
}

fn detect_inline_protocol() -> Option<InlineProtocol> {
    if let Ok(value) = std::env::var("DSC_EMOJI_INLINE_PROTOCOL") {
        let value = value.trim().to_ascii_lowercase();
        if value == "iterm2" || value == "iterm" {
            return Some(InlineProtocol::Iterm2);
        }
        if value == "kitty" {
            return Some(InlineProtocol::Kitty);
        }
        if value == "off" || value == "0" {
            return None;
        }
    }
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program == "iTerm.app" || term_program == "WezTerm" {
            return Some(InlineProtocol::Iterm2);
        }
    }
    if std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("KITTY_SESSION_ID").is_ok()
        || std::env::var("TERM")
            .map(|t| t.contains("kitty"))
            .unwrap_or(false)
    {
        return Some(InlineProtocol::Kitty);
    }
    None
}

fn print_inline_emojis(emojis: &[crate::api::CustomEmoji], protocol: InlineProtocol) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    for emoji in emojis {
        let image = client.get(&emoji.url).send();
        let image = match image {
            Ok(response) if response.status().is_success() => response.bytes(),
            _ => {
                println!("{}\t{}", emoji.name, emoji.url);
                continue;
            }
        };
        let image = match image {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("{}\t{}", emoji.name, emoji.url);
                continue;
            }
        };
        let encoded = base64::engine::general_purpose::STANDARD.encode(&image);
        match protocol {
            InlineProtocol::Iterm2 => {
                let sequence = format!(
                    "\u{1b}]1337;File=inline=1;width=1;height=1;preserveAspectRatio=1:{}\u{7}",
                    encoded
                );
                println!("{} {}", emoji.name, sequence);
            }
            InlineProtocol::Kitty => {
                let sequence = format!("\u{1b}_Gf=100,t=d;{}\u{1b}\\", encoded);
                println!("{} {}", emoji.name, sequence);
            }
        }
    }
    Ok(())
}

fn emoji_name_from_path(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("emoji path missing filename: {}", path.display()))?;
    let slug = slugify(stem);
    let name = slug.replace('-', "_");
    if name.is_empty() {
        return Err(anyhow!("emoji name is empty for {}", path.display()));
    }
    Ok(name)
}

fn is_emoji_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "svg"
    )
}

fn existing_emoji_names(client: &DiscourseClient) -> Result<HashSet<String>> {
    let emojis = client.list_custom_emojis()?;
    Ok(emojis
        .into_iter()
        .map(|emoji| emoji_key(&emoji.name))
        .collect())
}

fn emoji_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn is_duplicate_emoji_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("already been taken")
        || msg.contains("already exists")
        || msg.contains("emoji already exists")
}

#[cfg(test)]
mod tests {
    use super::is_duplicate_emoji_error;
    use anyhow::anyhow;

    #[test]
    fn duplicate_error_is_detected() {
        let err = anyhow!(
            "emoji upload failed with 422: {{\"errors\":[\"Name has already been taken\"]}}"
        );
        assert!(is_duplicate_emoji_error(&err));
    }

    #[test]
    fn non_duplicate_error_is_not_detected() {
        let err = anyhow!("emoji upload failed with 413: payload too large");
        assert!(!is_duplicate_emoji_error(&err));
    }
}
