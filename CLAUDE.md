# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is "assumer", a Terminal User Interface (TUI) application for AWS SSO authentication and role assumption. The application is built in Rust using the ratatui library for the terminal interface and the AWS SDK for Rust for AWS integration.

The application provides a complete workflow for AWS SSO:
1. Account listing and selection
2. Role selection within accounts
3. Credential generation and export
4. Browser-based console access (requires Granted Firefox extension)
5. AWS CLI profile management

## Architecture

The codebase follows a modular architecture with these key components:

### Core Modules
- `main.rs` - Entry point, initializes TUI and runs the main application loop
- `app.rs` - Central application state and routing system with a page-based navigation model
- `tui.rs` - Terminal initialization and restoration using crossterm/ratatui
- `sso.rs` - AWS SSO operations and credential management
- `aws/` - AWS SDK integration components
- `widgets/` - UI components for different application pages
- `utils/` - Utility functions for JSON and serialization

### Application Flow
The app uses a state-driven routing system where different pages (AccountList, Config, Credentials, Roles) are rendered based on the application state. The main state transitions are:
- Start → AccountList (shows available AWS accounts)
- AccountList → Roles (shows roles for selected account)
- Roles → Credentials (displays and manages credentials for selected role)
- Any page → Config (configuration management)

### Key Data Structures
- `App` - Main application state containing all UI state, selected items, and configuration
- `AccountRow` - Represents an AWS account with its roles
- `RoleCredentials` - AWS temporary credentials for a specific role
- `ConfigProvider` - Manages AWS SDK configuration and token providers

## Common Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build release version
cargo build --release
```

### Development
```bash
# Check code without building
cargo check

# Run tests (if any)
cargo test

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Clean build artifacts
cargo clean
```

### Installation
The project can be installed via Homebrew:
```bash
brew tap jrivers-iclass/tools
brew install assumer
```

## Configuration

The application stores configuration in `~/.assumer/config.ini` with these key settings:
- `start_url` - AWS SSO start URL (required)
- `aws_config_path` - Path to AWS configuration directory (defaults to ~/.aws)
- `region` - AWS region (defaults to us-east-1)

AWS profiles are exported to `~/.aws/config` with the format `assumer-{account_name}/{role_name}`.

## Dependencies

Key external dependencies:
- `ratatui` - Terminal UI framework
- `aws-config`, `aws-sdk-sso`, `aws-sdk-ssooidc` - AWS SDK components
- `tokio` - Async runtime
- `anyhow`/`color-eyre` - Error handling
- `serde`/`serde_json` - Serialization
- `directories` - Cross-platform directory handling
- `rust-ini` - INI file configuration

## Browser Integration

The application integrates with Firefox through the Granted extension for console access. It generates federated sign-in URLs and opens them using platform-specific commands:
- macOS: `open -na Firefox`
- Windows: PowerShell with `Start-Process firefox`
- Linux: Direct `firefox` command