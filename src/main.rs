mod config;
mod update;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
use std::process::Command;

#[derive(Parser)]
#[command(name = "dusk-warden", about = "Sync Bitwarden secrets to .env files")]
enum Cli {
    /// Download secrets from Bitwarden and write them to configured .env files
    Sync,
    /// Update dusk-warden to the latest release
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Sync => sync()?,
        Cli::Update => update::update()?,
    }

    Ok(())
}

fn sync() -> Result<()> {
    let config = Config::load()?;
    let token = config::resolve_bws_token()?;

    for secret in &config.secrets {
        let value = fetch_secret(&secret.id, &token)
            .with_context(|| format!("Failed to fetch secret for {}", secret.path))?;

        std::fs::write(&secret.path, value)
            .with_context(|| format!("Failed to write {}", secret.path))?;

        println!("Wrote {}", secret.path);
    }

    Ok(())
}

fn fetch_secret(secret_id: &str, token: &str) -> Result<String> {
    let output = Command::new("bws")
        .args(["secret", "get", secret_id])
        .env("BWS_ACCESS_TOKEN", token)
        .output()
        .context("Failed to run bws CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("bws failed: {stderr}");
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse bws output as JSON")?;

    json["value"]
        .as_str()
        .map(|s| s.to_string())
        .context("Secret value not found in bws output")
}
