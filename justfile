default:
    @just --list

# Run the CLI
run *args:
    cargo run -- {{args}}

# Build in release mode
build:
    cargo build --release

# Check compilation without building
check:
    cargo check

# Run tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt -- --check

# Release a new version using cargo-release
# Requires: cargo install cargo-release
# Usage:
#   just release patch   # 0.2.0 -> 0.2.1
#   just release minor   # 0.2.0 -> 0.3.0
#   just release major   # 0.2.0 -> 1.0.0
#   just release 0.3.0   # explicit version
# Dry run (default) shows what would happen without making changes.
# Pass --execute to actually perform the release:
#   just release patch --execute
release *args:
    cargo release {{args}}
