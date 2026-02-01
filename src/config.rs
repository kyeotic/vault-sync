use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

const CONFIG_FILE: &str = ".dusk-warden.toml";
const BWS_FILE: &str = ".bws";

#[derive(Deserialize)]
pub struct Config {
    pub secrets: Vec<SecretMapping>,
}

#[derive(Deserialize)]
pub struct SecretMapping {
    pub id: String,
    pub path: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string(CONFIG_FILE)
            .with_context(|| format!("Could not read {CONFIG_FILE}"))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse {CONFIG_FILE}"))
    }
}

pub fn resolve_bws_token() -> Result<String> {
    if let Ok(token) = std::env::var("BWS_ACCESS_TOKEN")
        && !token.is_empty()
    {
        return Ok(token);
    }

    let home = PathBuf::from(std::env::var("HOME").context("HOME not set")?);
    let mut dir = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        let candidate = dir.join(BWS_FILE);
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate)
                .with_context(|| format!("Failed to read {}", candidate.display()))?;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=')
                    && key.trim() == "BWS_ACCESS_TOKEN"
                {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    if !value.is_empty() {
                        return Ok(value.to_string());
                    }
                }
            }

            anyhow::bail!(
                "Found {} but it does not contain BWS_ACCESS_TOKEN",
                candidate.display()
            );
        }

        if dir == home || !dir.starts_with(&home) {
            break;
        }
        if !dir.pop() {
            break;
        }
    }

    anyhow::bail!(
        "BWS_ACCESS_TOKEN not found. Set it as an environment variable or in a {BWS_FILE} file."
    )
}
