use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oidc-cli")]
#[command(about = "A CLI tool for OAuth 2.0/OpenID Connect authentication")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Authenticate using a profile")]
    Login {
        #[arg(help = "Profile name to use for authentication")]
        profile: Option<String>,

        #[arg(short, long, help = "Port for the callback server")]
        port: Option<u16>,

        #[arg(long, help = "Copy tokens to clipboard")]
        copy: bool,

        #[arg(long, help = "Output tokens as JSON", action = ArgAction::SetTrue)]
        json: bool,

        #[arg(
            short = 'o',
            long,
            value_name = "FILE",
            help = "Write token output to file (implies --json)"
        )]
        output: Option<PathBuf>,
    },

    #[command(about = "List all available profiles")]
    List,

    #[command(about = "Create a new profile")]
    Create {
        #[arg(help = "Name of the new profile")]
        name: String,

        #[arg(long, help = "Client ID")]
        client_id: Option<String>,

        #[arg(long, help = "Client secret (optional)")]
        client_secret: Option<String>,

        #[arg(long, help = "Redirect URI")]
        redirect_uri: Option<String>,

        #[arg(long, help = "OAuth scope")]
        scope: Option<String>,

        #[arg(long, help = "OIDC discovery URI")]
        discovery_uri: Option<String>,

        #[arg(long, help = "Authorization endpoint (if not using discovery)")]
        auth_endpoint: Option<String>,

        #[arg(long, help = "Token endpoint (if not using discovery)")]
        token_endpoint: Option<String>,

        #[arg(long, help = "Non-interactive mode (requires all parameters)")]
        non_interactive: bool,
    },

    #[command(about = "Edit an existing profile")]
    Edit {
        #[arg(help = "Name of the profile to edit")]
        name: String,
    },

    #[command(about = "Delete a profile")]
    Delete {
        #[arg(help = "Name of the profile to delete")]
        name: String,

        #[arg(short, long, help = "Skip confirmation prompt")]
        force: bool,
    },

    #[command(about = "Rename a profile")]
    Rename {
        #[arg(help = "Current profile name")]
        old_name: String,

        #[arg(help = "New profile name")]
        new_name: String,
    },

    #[command(about = "Export profiles to a file")]
    Export {
        #[arg(help = "Output file path")]
        file: PathBuf,

        #[arg(help = "Specific profile names to export (exports all if not specified)")]
        profiles: Vec<String>,
    },

    #[command(about = "Import profiles from a file")]
    Import {
        #[arg(help = "Input file path")]
        file: PathBuf,

        #[arg(short, long, help = "Overwrite existing profiles")]
        overwrite: bool,
    },
}

impl Cli {
    pub fn is_verbose(&self) -> bool {
        self.verbose && !self.quiet
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_args() {
        Cli::command().debug_assert()
    }

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(["oidc-cli", "login", "test-profile"]).unwrap();

        match cli.command {
            Commands::Login { profile, .. } => {
                assert_eq!(profile, Some("test-profile".to_string()));
            }
            _ => panic!("Expected Login command"),
        }
    }

    #[test]
    fn test_verbose_quiet_flags() {
        let cli = Cli::try_parse_from(["oidc-cli", "--verbose", "list"]).unwrap();
        assert!(cli.is_verbose());
        assert!(!cli.is_quiet());

        let cli = Cli::try_parse_from(["oidc-cli", "--quiet", "list"]).unwrap();
        assert!(!cli.is_verbose());
        assert!(cli.is_quiet());

        let cli = Cli::try_parse_from(["oidc-cli", "--verbose", "--quiet", "list"]).unwrap();
        assert!(!cli.is_verbose());
        assert!(cli.is_quiet());
    }
}
