mod auth;
mod browser;
mod cli;
mod commands;
mod config;
mod crypto;
mod error;
mod profile;
mod server;
mod ui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use commands::*;
use error::{OidcError, Result};
use profile::ProfileManager;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        if !matches!(e, OidcError::Cancelled) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    let mut profile_manager = ProfileManager::new()?;

    let is_quiet = cli.is_quiet();
    let is_verbose = cli.is_verbose();

    match cli.command {
        Commands::Login {
            profile,
            port,
            copy,
        } => handle_login(profile_manager, profile, port, copy, is_quiet, is_verbose).await,
        Commands::List => handle_list(profile_manager, is_quiet),
        Commands::Create {
            name,
            client_id,
            client_secret,
            redirect_uri,
            scope,
            discovery_uri,
            auth_endpoint,
            token_endpoint,
            non_interactive,
        } => {
            handle_create(
                &mut profile_manager,
                CreateParams {
                    name,
                    client_id,
                    client_secret,
                    redirect_uri,
                    scope,
                    discovery_uri,
                    auth_endpoint,
                    token_endpoint,
                    non_interactive,
                    quiet: is_quiet,
                },
            )
            .await
        }
        Commands::Edit { name } => handle_edit(&mut profile_manager, name, is_quiet).await,
        Commands::Delete { name, force } => {
            handle_delete(&mut profile_manager, name, force, is_quiet)
        }
        Commands::Rename { old_name, new_name } => {
            handle_rename(&mut profile_manager, old_name, new_name, is_quiet)
        }
        Commands::Export { file, profiles } => {
            handle_export(profile_manager, file, profiles, is_quiet)
        }
        Commands::Import { file, overwrite } => {
            handle_import(&mut profile_manager, file, overwrite, is_quiet)
        }
    }
}