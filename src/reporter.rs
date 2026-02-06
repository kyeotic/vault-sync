use crate::styles::{AnsiPadding, AppStyles};

pub struct Reporter;

impl Reporter {
    // --- sync/push action labels ---

    pub fn would_update(path: &str) {
        println!(
            " {} {}",
            "Would update".would_update().align_right(12),
            path
        );
    }

    pub fn updated(path: &str) {
        println!(" {} {}", "Updated".updated().align_right(12), path);
    }

    pub fn up_to_date(path: &str) {
        println!(
            " {} {}",
            "Up to date".up_to_date().align_right(12),
            path
        );
    }

    pub fn pushed(path: &str) {
        println!(" {} {}", "Pushed".updated().align_right(12), path);
    }

    // --- update command labels ---

    pub fn current_version(version: &str) {
        println!(
            " {} {}",
            "Current".field_label().align_right(12),
            version
        );
    }

    pub fn latest_version(version: &str) {
        println!(
            " {} {}",
            "Latest".field_label().align_right(12),
            version
        );
    }

    pub fn already_up_to_date() {
        println!(
            " {}",
            "Already up to date.".up_to_date()
        );
    }

    pub fn downloading(url: &str) {
        println!(
            " {} {}",
            "Downloading".waiting().align_right(12),
            url
        );
    }

    pub fn self_updated(version: &str) {
        println!(
            " {} to {}",
            "Updated".updated().align_right(12),
            version
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Reporter;

    #[test]
    fn style_gallery() {
        println!();
        println!("  === Sync actions ===");
        Reporter::would_update("services/api/.env");
        Reporter::updated("services/api/.env");
        Reporter::up_to_date("services/api/.env");
        Reporter::pushed("services/api/.env");
        println!();
        println!("  === Update actions ===");
        Reporter::current_version("0.5.4");
        Reporter::latest_version("0.6.0");
        Reporter::already_up_to_date();
        Reporter::downloading("https://github.com/kyeotic/vault-sync/releases/download/v0.6.0/vault-sync-x86_64-unknown-linux-musl.tar.gz");
        Reporter::self_updated("0.6.0");
    }
}
