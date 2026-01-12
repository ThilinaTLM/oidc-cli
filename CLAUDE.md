# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Build
cargo build                    # Development build
cargo build --release          # Optimized release build

# Run
cargo run -- <command> <args>  # Run with arguments (e.g., cargo run -- login my-profile)

# Test
cargo test                     # Run all tests
cargo test <test_name>         # Run specific test
cargo test -- --nocapture      # Show test output

# Lint and Format
cargo fmt --all -- --check     # Check formatting
cargo fmt                      # Fix formatting
cargo clippy --all-targets --all-features --locked -- -D warnings  # Lint
```

## Architecture Overview

This is a Rust CLI tool for OAuth 2.0/OIDC authentication with PKCE support.

### Module Structure

```
src/
├── main.rs           # Entry point, command dispatch
├── cli.rs            # Clap CLI definitions
├── config.rs         # Profile/Config structures
├── error.rs          # OidcError enum (thiserror)
├── crypto.rs         # PKCE generation (SHA256, 256-bit entropy)
├── browser.rs        # BrowserOpener trait + WebBrowserOpener/MockBrowserOpener
├── server.rs         # HTTP callback server for OAuth redirects
├── auth/
│   ├── oauth.rs      # OAuthClient: auth requests, token exchange
│   ├── discovery.rs  # OIDC discovery endpoint parsing
│   └── pkce.rs       # PKCE challenge/verifier
├── profile/
│   ├── manager.rs    # ProfileManager CRUD operations
│   ├── storage.rs    # File I/O, JSON serialization
│   └── validation.rs # Input validation, sanitization
├── commands/
│   ├── login.rs      # OAuth flow orchestration
│   ├── profile.rs    # Profile CRUD commands
│   └── import_export.rs
└── ui/
    ├── prompts.rs    # Interactive prompts
    ├── display.rs    # Token/profile display
    └── manual_entry.rs
```

### Key Abstractions

**OAuth Flow**: `OAuthClient` creates authorization requests with PKCE → `CallbackServer` handles redirects via HTTP on localhost → token exchange completes in `OAuthClient::exchange_code_for_tokens()`

**Browser Integration**: `BrowserOpener` trait enables testing via `MockBrowserOpener` (tracks opened URLs). Production uses `WebBrowserOpener`.

**Profile System**: Two modes - discovery-based (auto-resolves endpoints from OIDC discovery URI) or manual (explicit authorization/token endpoints). Stored as JSON with restricted file permissions.

**Error Handling**: All functions return `Result<T, OidcError>`. Error types defined in `error.rs` using `thiserror`.

### Testing Patterns

- Unit tests embedded in modules with `#[cfg(test)]`
- Integration tests in `tests/` directory
- `MockBrowserOpener` for browser abstraction testing
- `ProfileManager` accepts test directory override for isolated tests
- Test environment variables: `OIDC_CLI_TEST_MODE`, `OIDC_CLI_TEST_DIR`

### Optional Features

- `clipboard` feature: enables `--copy` flag for copying tokens (`cargo build --features clipboard`)
