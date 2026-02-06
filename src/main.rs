mod config;
mod reporter;
mod styles;
mod update;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
use rayon::prelude::*;
use reporter::Reporter;
use std::process::Command;

#[derive(Parser)]
#[command(name = "vault-sync", about = "Sync Bitwarden secrets to .env files")]
enum Cli {
    /// Download secrets from Bitwarden and write them to configured .env files
    Sync {
        /// Show what would change without writing files
        #[arg(long, visible_alias = "check")]
        dry_run: bool,
    },
    /// Upload local .env files to Bitwarden secrets
    Push,
    /// Update vault-sync to the latest release
    Update,
    /// Print version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Sync { dry_run } => sync(dry_run)?,
        Cli::Push => push()?,
        Cli::Update => update::update()?,
        Cli::Version => println!("vault-sync {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}

fn sync(dry_run: bool) -> Result<()> {
    let config = Config::load()?;
    let token = config::resolve_bws_token()?;

    // Fetch all secrets in parallel (the slow part)
    let results: Vec<_> = config
        .secrets
        .par_iter()
        .map(|secret| {
            let value = fetch_secret(&secret.id, &token)
                .with_context(|| format!("Failed to fetch secret for {}", secret.path))?;
            let existing = std::fs::read_to_string(&secret.path).ok();
            Ok((secret, value, existing))
        })
        .collect::<Result<Vec<_>>>()?;

    // Process results sequentially (file writes and output)
    for (secret, value, existing) in results {
        let changed = existing.as_ref() != Some(&value);

        if changed {
            if dry_run {
                Reporter::would_update(&secret.path);
            } else {
                std::fs::write(&secret.path, &value)
                    .with_context(|| format!("Failed to write {}", secret.path))?;
                Reporter::updated(&secret.path);
            }
        } else {
            Reporter::up_to_date(&secret.path);
        }
    }

    Ok(())
}

fn push() -> Result<()> {
    let config = Config::load()?;
    let token = config::resolve_bws_token()?;

    for secret in &config.secrets {
        let value = std::fs::read_to_string(&secret.path)
            .with_context(|| format!("Failed to read {}", secret.path))?;

        update_secret(&secret.id, &value, &token)
            .with_context(|| format!("Failed to push secret for {}", secret.path))?;

        Reporter::pushed(&secret.path);
    }

    Ok(())
}

fn update_secret(secret_id: &str, value: &str, token: &str) -> Result<()> {
    let output = Command::new("bws")
        .args(["secret", "edit", "--value", value, secret_id])
        .env("BWS_ACCESS_TOKEN", token)
        .output()
        .context("Failed to run bws CLI. Is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(check_bws_error(&stderr, secret_id));
    }

    Ok(())
}

fn check_bws_error(stderr: &str, secret_id: &str) -> anyhow::Error {
    if stderr.contains("404") || stderr.contains("Resource not found") {
        anyhow::anyhow!(
            "Secret {secret_id} not found or access denied. \
             Check that your service account token has write permissions."
        )
    } else {
        anyhow::anyhow!("bws failed: {stderr}")
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_bws_error_returns_permission_message_on_404() {
        let stderr = "Error: Received error message from server: [404 Not Found] {\"message\":\"Resource not found.\"}";
        let err = check_bws_error(stderr, "abc-123");
        let msg = err.to_string();
        assert!(msg.contains("not found or access denied"), "got: {msg}");
        assert!(msg.contains("write permissions"), "got: {msg}");
        assert!(msg.contains("abc-123"), "got: {msg}");
    }

    #[test]
    fn check_bws_error_returns_raw_stderr_for_other_errors() {
        let stderr = "Error: something else went wrong";
        let err = check_bws_error(stderr, "abc-123");
        let msg = err.to_string();
        assert!(msg.contains("bws failed:"), "got: {msg}");
        assert!(msg.contains("something else went wrong"), "got: {msg}");
    }
}
