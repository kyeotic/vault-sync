use anyhow::{Context, Result};
use serde::Deserialize;

const CONFIG_FILE: &str = ".dusk-warden.toml";

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
