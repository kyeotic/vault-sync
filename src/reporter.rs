use crate::config::SecretMapping;
use crate::styles::{AnsiPadding, AppStyles};

pub struct Reporter;

impl Reporter {
    // --- sync/push action labels ---

    fn format_secret(secret: &SecretMapping) -> String {
        format!("{} {}", secret.name, format!("({})", secret.path).dimmed())
    }

    pub fn would_update(secret: &SecretMapping) {
        println!(
            " {} {}",
            "Would update".would_update().align_right(12),
            Self::format_secret(secret)
        );
    }

    pub fn updated(secret: &SecretMapping) {
        println!(
            " {} {}",
            "Updated".updated().align_right(12),
            Self::format_secret(secret)
        );
    }

    pub fn up_to_date(secret: &SecretMapping) {
        println!(
            " {} {}",
            "Up to date".up_to_date().align_right(12),
            Self::format_secret(secret)
        );
    }

    pub fn pushed(secret: &SecretMapping) {
        println!(
            " {} {}",
            "Pushed".updated().align_right(12),
            Self::format_secret(secret)
        );
    }

    pub fn retrying(secret: &SecretMapping, attempt: u32, max_attempts: u32) {
        println!(
            " {} {} (429, attempt {}/{})",
            "Retrying".waiting().align_right(12),
            Self::format_secret(secret),
            attempt,
            max_attempts
        );
    }

    // --- update command labels ---

    pub fn current_version(version: &str) {
        println!(" {} {}", "Current".field_label().align_right(12), version);
    }

    pub fn latest_version(version: &str) {
        println!(" {} {}", "Latest".field_label().align_right(12), version);
    }

    pub fn already_up_to_date() {
        println!(" {}", "Already up to date.".up_to_date());
    }

    pub fn downloading(url: &str) {
        println!(" {} {}", "Downloading".waiting().align_right(12), url);
    }

    pub fn self_updated(version: &str) {
        println!(" {} to {}", "Updated".updated().align_right(12), version);
    }
}

#[cfg(test)]
mod tests {
    use super::Reporter;
    use crate::config::SecretMapping;

    #[test]
    fn style_gallery() {
        let secret = SecretMapping {
            name: "api".to_string(),
            id: "fake-id".to_string(),
            path: "services/api/.env".to_string(),
        };
        println!();
        println!("  === Sync actions ===");
        Reporter::would_update(&secret);
        Reporter::updated(&secret);
        Reporter::up_to_date(&secret);
        Reporter::pushed(&secret);
        Reporter::retrying(&secret, 1, 3);
        println!();
        println!("  === Update actions ===");
        Reporter::current_version("0.5.4");
        Reporter::latest_version("0.6.0");
        Reporter::already_up_to_date();
        Reporter::downloading(
            "https://github.com/kyeotic/vault-sync/releases/download/v0.6.0/vault-sync-x86_64-unknown-linux-musl.tar.gz",
        );
        Reporter::self_updated("0.6.0");
    }
}
