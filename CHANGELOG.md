# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.3.0]

### Added

- `.bws` file lookup for `BWS_ACCESS_TOKEN`. Searches from the current directory upward to `$HOME`, falling back to the environment variable.

## [0.1.0]

### Added

- `sync` command to download Bitwarden secrets and write them to configured `.env` files.
- `update` command for in-place self-update from GitHub releases.
- Install script for quick setup.
