# OIDC CLI

A command-line tool for OAuth 2.0/OpenID Connect authentication with PKCE support.

[![CI](https://github.com/ThilinaTLM/oidc-cli/actions/workflows/ci.yaml/badge.svg)](https://github.com/ThilinaTLM/oidc-cli/actions/workflows/ci.yaml)
[![Release](https://img.shields.io/github/v/release/ThilinaTLM/oidc-cli)](https://github.com/ThilinaTLM/oidc-cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [JSON Export](#json-export)
- [Configuration](#configuration)
- [Security](#security)
- [Examples](#examples)
- [Development](#development)
- [License](#license)

## Features

- **OAuth 2.0/OIDC** - Authorization Code flow with PKCE (RFC 7636)
- **Profile Management** - Create, edit, delete, rename, import/export profiles
- **Auto-Discovery** - Automatic endpoint resolution via OIDC discovery
- **JSON Export** - Export tokens to stdout or file with `--json` / `--output`
- **Cross-Platform** - Linux, macOS, and Windows support
- **Scriptable** - Quiet mode for CI/CD pipelines and scripts

## Demo

https://github.com/user-attachments/assets/55bb54a5-470e-41f3-ace7-dac2110728b2

## Installation

### Linux

```bash
curl -L -o oidc-cli https://github.com/ThilinaTLM/oidc-cli/releases/latest/download/oidc-cli-x86_64-unknown-linux-gnu
chmod +x oidc-cli
sudo mv oidc-cli /usr/local/bin/
```

### macOS

```bash
curl -L -o oidc-cli https://github.com/ThilinaTLM/oidc-cli/releases/latest/download/oidc-cli-x86_64-apple-darwin
chmod +x oidc-cli
sudo mv oidc-cli /usr/local/bin/
```

### Windows

```powershell
curl -L -o oidc-cli.exe https://github.com/ThilinaTLM/oidc-cli/releases/latest/download/oidc-cli-x86_64-pc-windows-msvc.exe
```

### Build from Source

```bash
cargo build --release
# Binary at target/release/oidc-cli
```

## Quick Start

```bash
# 1. Create a profile
oidc-cli create my-profile

# 2. Authenticate
oidc-cli login my-profile

# 3. Export token as JSON
oidc-cli login my-profile --json
```

## Usage

### Authentication

```bash
oidc-cli login [PROFILE]           # Login (auto-selects if one profile)
oidc-cli login my-profile          # Login with specific profile
oidc-cli login my-profile -p 9000  # Custom callback port
oidc-cli login my-profile --copy   # Copy access token to clipboard
```

### JSON Export

```bash
oidc-cli login my-profile --json              # JSON to stdout
oidc-cli login my-profile -o tokens.json      # JSON to file
oidc-cli login my-profile --output tokens.json
```

Output format:

```json
{
  "access_token": "eyJ...",
  "token_type": "Bearer",
  "expires_at": 1736712000,
  "refresh_token": "...",
  "id_token": "eyJ...",
  "scope": "openid profile email"
}
```

> Note: `expires_at` is a Unix timestamp (absolute), not relative seconds.

### Profile Management

```bash
oidc-cli create <name>              # Create profile (interactive)
oidc-cli list                       # List all profiles
oidc-cli edit <name>                # Edit profile
oidc-cli delete <name>              # Delete profile
oidc-cli delete <name> --force      # Delete without confirmation
oidc-cli rename <old> <new>         # Rename profile
```

### Import/Export

```bash
oidc-cli export profiles.json                  # Export all profiles
oidc-cli export profiles.json profile1 profile2  # Export specific profiles
oidc-cli import profiles.json                  # Import profiles
oidc-cli import profiles.json --overwrite      # Overwrite existing
```

### Global Options

| Option      | Description                    |
|-------------|--------------------------------|
| `--verbose` | Show detailed output           |
| `--quiet`   | Minimal output (for scripting) |
| `--help`    | Show help                      |
| `--version` | Show version                   |

## Configuration

Profiles are stored in your system config directory and support two modes:

### Discovery-based (Recommended)

```json
{
  "discovery_uri": "https://auth.example.com/.well-known/openid-configuration",
  "client_id": "your-client-id",
  "client_secret": "optional-secret",
  "redirect_uri": "http://localhost:8080/callback",
  "scope": "openid profile email"
}
```

### Manual Endpoints

```json
{
  "client_id": "your-client-id",
  "redirect_uri": "http://localhost:8080/callback",
  "scope": "openid profile email",
  "authorization_endpoint": "https://auth.example.com/authorize",
  "token_endpoint": "https://auth.example.com/token"
}
```

## Security

| Feature             | Implementation                              |
|---------------------|---------------------------------------------|
| PKCE                | SHA256 code challenge, 256-bit entropy      |
| State Parameter     | CSRF protection, 128-bit entropy            |
| Input Validation    | All inputs validated and sanitized          |
| File Permissions    | Profile files stored with restricted access |
| Token Storage       | Tokens are never persisted to disk          |

## Examples

### Non-interactive Profile Creation

```bash
oidc-cli create google \
  --client-id "your-client-id" \
  --redirect-uri "http://localhost:8080/callback" \
  --scope "openid profile email" \
  --discovery-uri "https://accounts.google.com/.well-known/openid-configuration" \
  --non-interactive
```

### Scripting with JSON Export

```bash
# Get access token
TOKEN=$(oidc-cli login my-profile --json | jq -r '.access_token')

# Use in API call
curl -H "Authorization: Bearer $TOKEN" https://api.example.com/user

# Save tokens to file
oidc-cli login my-profile -o /tmp/tokens.json
```

### Check Token Expiration

```bash
# Get expiration timestamp
EXPIRES=$(oidc-cli login my-profile --json | jq '.expires_at')

# Convert to human-readable
date -d @$EXPIRES  # Linux
date -r $EXPIRES   # macOS
```

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for architecture details and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
