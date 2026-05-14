# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

"assumer" is a Terminal User Interface (TUI) for AWS SSO authentication and role assumption, built in Rust with ratatui and the AWS SDK for Rust.

Workflow: authenticate via SSO → select account → select role → export credentials or open browser console.

## Commands

```bash
cargo build           # debug build
cargo build --release # release build
cargo run             # run (requires ~/.assumer/config.ini with start_url)
cargo check           # fast type-check without linking
cargo clippy          # lint
cargo fmt             # format
```

Debug logging (logs go to stderr — redirect to avoid corrupting the TUI):
```bash
RUST_LOG=debug cargo run 2>/tmp/assumer.log
```

## Architecture

### State and routing (`app.rs`)
`App` is the single shared mutable state passed everywhere. `CurrentPage` is an enum (`AccountList`, `Config`, `Credentials`, `Roles`) that drives which widget renders. Routes are stored in `App.routes: HashMap<CurrentPage, RouteConfig>` where each entry holds a layout fn and a render fn. Navigation is purely state mutation — there is no router framework.

### Async boundary (`sso.rs`)
All AWS SDK calls are async, but the ratatui event loop is synchronous. Each public function in `sso.rs` is annotated `#[tokio::main]`, making it a blocking call from the perspective of `app.rs`. This means AWS calls block the render loop while running — there is no background task or channel-based concurrency.

### AWS module (`src/aws/`)
- `token.rs` — `SsoAccessTokenProvider`: handles device authorization flow (OIDC), token refresh, and caches tokens to `~/.aws/sso/cache/<sha1(session_name)>.json`
- `token_cache.rs` — `AccessTokenCache`: reads/writes the JSON cache file; cache key is SHA1 of the SSO session name
- `account_info_provider.rs` — `AccountInfoProvider`: wraps `aws-sdk-sso` client; `get_account_list` paginates via `nextToken` in pages of 100 (AWS API limit)
- `cli.rs` — derives the SSO session name from the start URL subdomain (`sso-{subdomain}`)

### Token flow
1. On startup, `App::load_aws_config` builds a `ConfigProvider` with an `SsoAccessTokenProvider`
2. `get_access_token` checks the cache → refreshes if expired → triggers full device auth flow if refresh fails
3. Device auth opens a browser for the user to approve, then polls until approved

### Configuration
`~/.assumer/config.ini` (`[Main]` section):
- `start_url` — required; AWS SSO start URL
- `aws_config_path` — defaults to `~/.aws`
- `region` — defaults to `us-east-1`

AWS profiles written to `~/.aws/config` use the format `assumer-{account_name}/{role_name}`.

## Release Process

1. Bump `version` in `Cargo.toml`, commit and push to `main`
2. Create a GitHub release with a `vX.Y.Z` tag on `https://github.com/j-fulbright/rust-aws-sso-tui`
3. The `homebrew-release.yml` workflow fires automatically and updates `https://github.com/j-fulbright/homebrew-tools` with the new version and SHA256

Install via:
```bash
brew tap j-fulbright/tools
brew install assumer
```
