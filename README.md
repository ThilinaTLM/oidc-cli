# OIDC CLI Tool

A command-line application for OAuth 2.0/OpenID Connect authentication with PKCE support.

[![CI](https://github.com/ThilinaTLM/oidc-cli/actions/workflows/ci.yaml/badge.svg)](https://github.com/ThilinaTLM/oidc-cli/actions/workflows/ci.yaml)

## Video Demonstration

See the CLI in action with this quick demonstration showing profile creation and authentication:

[Example: Create Profile & Login](https://github.com/user-attachments/assets/55bb54a5-470e-41f3-ace7-dac2110728b2)

## Features

- **OAuth 2.0/OIDC Authentication** - Full support for Authorization Code flow with PKCE
- **Profile Management** - Create, edit, delete, and manage multiple authentication profiles
- **Discovery Support** - Automatic endpoint discovery via OIDC discovery URIs
- **Security First** - PKCE with SHA256, state parameter validation, secure random generation
- **Cross-platform** - Windows, macOS, and Linux support
- **Browser Integration** - Automatic browser opening with fallback support
- **Import/Export** - Backup and share profiles securely
- **Interactive & Scriptable** - Both interactive and quiet modes supported

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/oidc-cli` (or `oidc-cli.exe` on Windows).

## Quick Start

1. **Create a profile**:

```bash
oidc-cli create my-profile
```

2. **List profiles**:

```bash
oidc-cli list
```

3. **Authenticate**:

```bash
oidc-cli login my-profile
```

## Commands

### Profile Management

- `create <profile>` - Create a new profile (interactive)
- `list` - List all profiles
- `edit <profile>` - Edit an existing profile
- `delete <profile>` - Delete a profile
- `rename <old> <new>` - Rename a profile

### Authentication

- `login [profile]` - Start OAuth flow (auto-selects if only one profile)
- `login <profile> --port 9000` - Use custom callback port
- `login <profile> --copy` - Copy access token to clipboard

### Import/Export

- `export <file>` - Export all profiles
- `export <file> profile1 profile2` - Export specific profiles
- `import <file>` - Import profiles
- `import <file> --overwrite` - Import and overwrite existing profiles

### Options

- `--verbose` - Show detailed output
- `--quiet` - Minimal output (for scripting)
- `--help` - Show help

## Profile Configuration

Profiles support two configuration methods:

### 1. Discovery-based (Recommended)

Uses OIDC discovery to automatically find endpoints:

```json
{
  "discovery_uri": "https://auth.example.com/.well-known/openid-configuration",
  "client_id": "your-client-id",
  "client_secret": "optional-secret",
  "redirect_uri": "http://localhost:8080/callback",
  "scope": "openid profile email"
}
```

### 2. Manual Endpoints

Specify endpoints manually:

```json
{
  "client_id": "your-client-id",
  "client_secret": "optional-secret",
  "redirect_uri": "http://localhost:8080/callback",
  "scope": "openid profile email",
  "authorization_endpoint": "https://auth.example.com/authorize",
  "token_endpoint": "https://auth.example.com/token"
}
```

## Security Features

- **PKCE (RFC 7636)**: SHA256 code challenge with 256-bit entropy
- **State Parameter**: CSRF protection with 128-bit entropy
- **Input Validation**: All inputs are validated and sanitized
- **Secure Storage**: Profile files stored with restricted permissions
- **No Token Persistence**: Tokens are never stored on disk

## Examples

### Interactive Profile Creation

```bash
$ oidc-cli create github
Creating new profile 'github'
Press Ctrl+C to cancel at any time

Client ID: your-github-client-id
Client Secret (optional):
Redirect URI [http://localhost:8080/callback]:
Scope [openid profile email]: user:email

Choose configuration method:
  1. Use discovery URI (recommended)
  2. Manual endpoint configuration
Select option (1-2): 2

Authorization Endpoint: https://github.com/login/oauth/authorize
Token Endpoint: https://github.com/login/oauth/access_token

✓ Profile 'github' created successfully!
```

### Non-interactive Profile Creation

```bash
oidc-cli create google \
  --client-id "your-google-client" \
  --redirect-uri "http://localhost:8080/callback" \
  --scope "openid profile email" \
  --discovery-uri "https://accounts.google.com/.well-known/openid-configuration" \
  --non-interactive
```

### Authentication Flow

```bash
$ oidc-cli login github
Initiating OAuth 2.0 authorization flow...
Opening browser for authentication...
Waiting for authentication callback...
Press Ctrl+C to cancel

🎉 Authentication successful!

Access Token: ya29.a0AfH6SMC...
Token Type: Bearer
Expires In: 3599 seconds
Scope: user:email
```

### Scripting Support

```bash
# Get just the token response as JSON
ACCESS_TOKEN=$(oidc-cli login github --quiet | jq -r '.access_token')

# Use in API calls
curl -H "Authorization: Bearer $ACCESS_TOKEN" https://api.github.com/user
```

## Development

For developers interested in contributing or understanding the technical details, see [DEVELOPMENT.md](DEVELOPMENT.md).

## Repository

GitHub: https://github.com/ThilinaTLM/oidc-cli

## License

MIT License
