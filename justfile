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
