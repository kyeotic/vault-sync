# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.6.0] - 2026-02-06

### Breaking Changes
- **Config format**: Secrets are now named maps instead of arrays. `[[secrets]]` becomes `[secrets.<name>]` with a unique name for each secret.
- **`update` command renamed to `upgrade`**: Use `vault-sync upgrade` instead of `vault-sync update`.

### Added
- Named secrets: each secret in `.vault-sync.toml` now has a unique name (the TOML table key).
- `push` accepts an optional positional argument to target a single secret by name (e.g., `vault-sync push api`).
- Reporter output now shows the secret name with a dimmed path for context.

## [0.5.8] - 2026-02-06

### Added
- Retry with exponential backoff for `bws` API calls that receive 429 (rate limit) responses.
- `max_threads` and `max_retries` config fields in `.vault-sync.toml` (default: 3 each).

### Changed
- `sync` now limits concurrent `bws` calls to `max_threads` (default 3) instead of using all CPU cores, preventing thundering herd 429 errors.

## [0.5.7] - 2026-02-05

### Added
- nix flake install instructions

### Changed
- `upgrade` handling when installed by nix

## [0.5.6] - 2026-02-05

### Changed
- Styled output for `update` command (version info, download progress, completion).
- Added `NO_COLOR` environment variable support for disabling colored output.

## [0.5.4] - 2026-02-03

### Added
- Homebrew tap release

## [0.5.3] - 2026-02-03

### Added

- Pretty printed output for `sync`.

### Changed

- `sync` now makes parallel calls to `bws` for secrets, instead of serial calls.

## [0.5.2] - 2026-02-03

### Added

- `version` command to print the current version.
- `--dry-run` option (with `--check` alias) for `sync` command to preview changes without writing files.
- Change detection in `sync` command — reports "up to date" or "updated" status for each file.

## [0.5.1] - 2026-02-03

### Added

- Template variable support in paths using `{{ env.VAR }}` syntax for environment variable interpolation.
- `version` command to print the current version.

## [0.5.0] - 2026-02-02

### Changed

- Project renamed from `dusk-warden` to `vault-sync`.
	- Binary/CLI command is now `vault-sync`.
	- Config file renamed to `.vault-sync.toml`.
	- Release artifacts and install script updated to `vault-sync`.
	- Documentation and GitHub workflow adjusted accordingly.

## [0.4.1] - 2026-02-01

### Fixed

- Improved error message when `push` fails due to missing write permissions on the service account token.

## [0.4.0] - 2026-02-01

### Added

- `push` command to upload local `.env` files to Bitwarden secrets.

## [0.3.0] - 2026-01-31

### Added

- `.bws` file lookup for `BWS_ACCESS_TOKEN`. Searches from the current directory upward to `$HOME`, falling back to the environment variable.

## [0.1.0] - 2026-01-31

### Added

- `sync` command to download Bitwarden secrets and write them to configured `.env` files.
- `update` command for in-place self-update from GitHub releases.
- Install script for quick setup.
