# dusk-warden

Sync [Bitwarden Secrets Manager](https://bitwarden.com/products/secrets-manager/) secrets to local `.env` files using the [Bitwarden Secrets CLI](https://bitwarden.com/help/secrets-manager-cli/).

## Prerequisites

- [Bitwarden Secrets CLI (`bws`)](https://bitwarden.com/help/secrets-manager-cli/) installed
- A `BWS_ACCESS_TOKEN` provided via environment variable or a `.bws` file (see below)

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/kyeotic/dusk-warden/main/install.sh | bash
```

Or download a binary from the [releases page](https://github.com/kyeotic/dusk-warden/releases).

## Access Token

dusk-warden needs a `BWS_ACCESS_TOKEN` to authenticate with the Bitwarden Secrets CLI. It resolves the token in this order:

1. **Environment variable** — if `BWS_ACCESS_TOKEN` is set, it is used directly.
2. **`.bws` file** — otherwise, dusk-warden searches for a `.bws` file starting from the current directory and walking up parent directories, stopping at `$HOME`.

A `.bws` file uses simple `KEY=value` format:

```
BWS_ACCESS_TOKEN=0.your-token-here
```

Quotes around the value are optional and will be stripped. This file should be added to your global `.gitignore` to avoid committing credentials.

## Quick Start

1. Create a `.dusk-warden.toml` config file in your project root:

```toml
[[secrets]]
id = "your-bitwarden-secret-id"
path = ".env"

[[secrets]]
id = "another-secret-id"
path = "services/api/.env"
```

Each entry maps a Bitwarden secret (by ID) to a local file path where its contents will be written.

2. Run the sync:

```bash
dusk-warden sync
```

This fetches each secret from Bitwarden and writes its value to the configured path.

## Secret Format

Each Bitwarden secret's **value** should contain the literal contents of the `.env` file it maps to. For example, a secret's value might be:

```
DATABASE_URL=postgres://localhost:5432/mydb
API_KEY=sk-abc123
DEBUG=true
```

The value is written to the target path exactly as-is, with no transformation. Structure your secrets as complete, ready-to-use `.env` files.

## Self-Update

```bash
dusk-warden update
```

This checks for the latest GitHub release and replaces the binary in-place if a newer version is available.
