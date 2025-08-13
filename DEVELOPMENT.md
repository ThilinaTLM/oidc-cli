# OIDC CLI Development Guide

This document contains technical details for developers working on the OIDC CLI tool.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run tests (comprehensive test suite with 33+ tests)
cargo test

# Run with specific profile
cargo run -- login <profile-name>

# Run in development mode
cargo run -- <command> <args>
```

## Architecture

The tool is built with a modular architecture:

### Core Modules Structure

- **`main.rs`** - Entry point with command dispatch and interactive prompts
- **`cli.rs`** - Clap-based CLI definitions and argument parsing
- **`config.rs`** - Configuration structures and validation
- **`error.rs`** - Centralized error handling with thiserror
- **`crypto.rs`** - PKCE cryptographic functions and secure random generation
- **`browser.rs`** - Cross-platform browser integration with fallback
- **`server.rs`** - HTTP callback server for OAuth redirects

### Authentication Module (`auth/`)

- **`oauth.rs`** - Core OAuth 2.0 client implementation with token exchange
- **`pkce.rs`** - PKCE code generation and verification (SHA256)
- **`discovery.rs`** - OIDC discovery endpoint parsing and validation
- **`mod.rs`** - Public API and type exports

### Profile Management (`profile/`)

- **`manager.rs`** - High-level profile CRUD operations
- **`storage.rs`** - File I/O and JSON serialization for profiles
- **`validation.rs`** - Profile configuration validation and sanitization
- **`mod.rs`** - Module exports and public interfaces

## Key Implementation Details

### OAuth Flow Architecture
The authentication flow spans multiple modules:
1. `OAuthClient` (auth/oauth.rs) creates authorization requests
2. `CallbackServer` (server.rs) handles OAuth redirects
3. PKCE challenge/verifier generated in `crypto.rs`
4. State validation prevents CSRF attacks
5. Token exchange happens in `OAuthClient::exchange_code_for_tokens()`

### Profile System
Profiles support two configuration modes:
1. **Discovery-based**: Uses OIDC discovery URI for automatic endpoint resolution
2. **Manual**: Requires explicit authorization and token endpoints

Profile files are stored in platform-specific user data directories with restricted permissions.

### Error Handling
Uses `thiserror` for structured error types with context. All functions return `Result<T>` with the custom `OidcError` enum.

### Security Features
- PKCE with SHA256 and 256-bit entropy (crypto.rs:15-25)
- State parameter with 128-bit entropy (crypto.rs:27-35)
- Input validation and sanitization in profile/validation.rs
- No token persistence to disk
- Secure file permissions for profiles

### CLI Design Patterns
- Interactive prompts with defaults and validation
- Non-interactive mode for scripting (`--non-interactive`)
- Quiet mode for JSON output (`--quiet`)
- Verbose mode for debugging (`--verbose`)
- Global flags handled in cli.rs:109-121

## Testing Strategy

The test suite covers:
- OAuth flow implementation (auth/ modules)
- PKCE cryptographic functions (crypto.rs)
- Profile management operations (profile/ modules)  
- Input validation and edge cases
- CLI argument parsing (cli.rs:123-159)
- Error handling scenarios

Run the comprehensive test suite:
```bash
cargo test
```

All 33 tests cover:
- OAuth flow implementation
- PKCE cryptographic functions
- Profile management
- Input validation
- Error handling
- CLI parsing

## Dependencies

- Uses Tokio for async HTTP operations
- Reqwest for HTTP client functionality
- Serde for JSON serialization
- Clap v4 with derive macros for CLI
- Platform-specific user data directories via `dirs` crate
- Cross-platform browser opening with `webbrowser` crate

## Development Notes

The codebase follows functional requirements specified in REQ.md with comprehensive coverage of 200+ requirements across authentication, profile management, CLI interface, security, and platform support.

## Contributing

When contributing to the project:

1. Follow Rust best practices and idioms
2. Ensure all tests pass with `cargo test`
3. Add tests for new functionality
4. Update documentation for user-facing changes
5. Use `cargo fmt` for consistent formatting
6. Run `cargo clippy` to catch common issues

## License

MIT License