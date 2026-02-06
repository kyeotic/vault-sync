use anyhow::{Context, Result};
use crate::reporter::Reporter;
use flate2::read::GzDecoder;
use std::io::Read;
use tar::Archive;

const REPO: &str = "kyeotic/vault-sync";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn get_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
        (os, arch) => anyhow::bail!("Unsupported platform: {os}/{arch}"),
    }
}

fn fetch_latest_tag(agent: &ureq::Agent) -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let body: serde_json::Value = agent
        .get(&url)
        .call()
        .context("Failed to fetch latest release")?
        .into_body()
        .read_json()
        .context("Failed to parse release JSON")?;

    body["tag_name"]
        .as_str()
        .map(|s: &str| s.to_string())
        .context("No tag_name in release response")
}

fn version_from_tag(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}

pub fn update() -> Result<()> {
    let agent = ureq::Agent::new_with_defaults();
    let latest_tag = fetch_latest_tag(&agent)?;
    let latest_version = version_from_tag(&latest_tag);

    Reporter::current_version(CURRENT_VERSION);
    Reporter::latest_version(latest_version);

    if latest_version == CURRENT_VERSION {
        Reporter::already_up_to_date();
        return Ok(());
    }

    let target = get_target()?;
    let url = format!(
        "https://github.com/{REPO}/releases/download/{latest_tag}/vault-sync-{target}.tar.gz"
    );

    Reporter::downloading(&url);
    let mut gz_bytes = Vec::new();
    agent
        .get(&url)
        .call()
        .context("Failed to download release")?
        .into_body()
        .as_reader()
        .read_to_end(&mut gz_bytes)
        .context("Failed to read response body")?;

    let decoder = GzDecoder::new(gz_bytes.as_slice());
    let mut archive = Archive::new(decoder);

    let binary = archive
        .entries()
        .context("Failed to read tar entries")?
        .find_map(|entry| {
            let mut entry = entry.ok()?;
            let path = entry.path().ok()?;
            if path.file_name()? == "vault-sync" {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).ok()?;
                Some(buf)
            } else {
                None
            }
        })
        .context("Binary not found in archive")?;

    // Write binary to a temp file, then use self_replace to atomically swap
    let tmp = std::env::temp_dir().join("vault-sync-update");
    std::fs::write(&tmp, &binary).context("Failed to write temp binary")?;
    self_replace::self_replace(&tmp).context("Failed to replace binary")?;
    let _ = std::fs::remove_file(&tmp);

    Reporter::self_updated(latest_version);
    Ok(())
}
