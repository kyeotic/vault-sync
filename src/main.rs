mod config;
mod reporter;
mod styles;
mod upgrade;

use anyhow::{Context, Result};
use clap::Parser;
use config::{Config, SecretMapping};
use rayon::prelude::*;
use reporter::Reporter;
use std::process::Command;
use std::thread;
use std::time::Duration;


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
    Push {
        /// Name of a specific secret to push (from config [secrets.<name>])
        target: Option<String>,
    },
    /// Upgrade vault-sync to the latest release
    Upgrade,
    /// Print version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Sync { dry_run } => sync(dry_run)?,
        Cli::Push { target } => push(target)?,
        Cli::Upgrade => upgrade::upgrade()?,
        Cli::Version => println!("vault-sync {}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}

fn sync(dry_run: bool) -> Result<()> {
    let config = Config::load()?;
    let token = config::resolve_bws_token()?;

    // Fetch all secrets in parallel with limited concurrency
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.max_threads)
        .build()
        .context("Failed to build thread pool")?;

    let entries: Vec<_> = config.secrets.values().collect();
    let results: Vec<_> = pool.install(|| {
        entries
            .par_iter()
            .map(|secret| {
                let value = fetch_secret(secret, &token, config.max_retries)
                    .with_context(|| format!("Failed to fetch secret '{}' for {}", secret.name, secret.path))?;
                let existing = std::fs::read_to_string(&secret.path).ok();
                Ok((*secret, value, existing))
            })
            .collect::<Result<Vec<_>>>()
    })?;

    // Process results sequentially (file writes and output)
    for (secret, value, existing) in results {
        let changed = existing.as_ref() != Some(&value);

        if changed {
            if dry_run {
                Reporter::would_update(secret);
            } else {
                std::fs::write(&secret.path, &value)
                    .with_context(|| format!("Failed to write {}", secret.path))?;
                Reporter::updated(secret);
            }
        } else {
            Reporter::up_to_date(secret);
        }
    }

    Ok(())
}

fn push(target: Option<String>) -> Result<()> {
    let config = Config::load()?;
    let token = config::resolve_bws_token()?;

    let secrets: Vec<_> = match &target {
        Some(name) => {
            let secret = config.secrets.get(name).with_context(|| {
                let available: Vec<_> = config.secrets.keys().collect();
                format!("Secret '{name}' not found in config. Available: {available:?}")
            })?;
            vec![secret]
        }
        None => config.secrets.values().collect(),
    };

    for secret in secrets {
        let value = std::fs::read_to_string(&secret.path)
            .with_context(|| format!("Failed to read {}", secret.path))?;

        update_secret(secret, &value, &token, config.max_retries)
            .with_context(|| format!("Failed to push secret '{}' for {}", secret.name, secret.path))?;

        Reporter::pushed(secret);
    }

    Ok(())
}

fn run_bws(args: &[&str], token: &str, secret: &SecretMapping, max_retries: u32) -> Result<std::process::Output> {
    for attempt in 1..=max_retries {
        let output = Command::new("bws")
            .args(args)
            .env("BWS_ACCESS_TOKEN", token)
            .output()
            .context("Failed to run bws CLI. Is it installed?")?;

        if output.status.success() {
            return Ok(output);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("429") && attempt < max_retries {
            Reporter::retrying(secret, attempt, max_retries);
            thread::sleep(Duration::from_secs(1 << attempt)); // 2s, 4s
            continue;
        }

        return Ok(output);
    }

    unreachable!()
}

fn update_secret(secret: &SecretMapping, value: &str, token: &str, max_retries: u32) -> Result<()> {
    let output = run_bws(&["secret", "edit", "--value", value, &secret.id], token, secret, max_retries)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(check_bws_error(&stderr, &secret.id));
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

fn fetch_secret(secret: &SecretMapping, token: &str, max_retries: u32) -> Result<String> {
    let output = run_bws(&["secret", "get", &secret.id], token, secret, max_retries)?;

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
