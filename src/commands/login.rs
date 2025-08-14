use crate::auth::OAuthClient;
use crate::browser::open_browser_with_fallback;
use crate::error::{OidcError, Result};
use crate::profile::ProfileManager;
use crate::server::CallbackServer;
use crate::ui::{display_tokens, handle_manual_code_entry, select_profile};
use crate::utils::url::{extract_port_from_redirect_uri, is_localhost_redirect_uri};
use tokio::time::{timeout, Duration};

pub async fn handle_login(
    profile_manager: ProfileManager,
    profile_name: Option<String>,
    port: Option<u16>,
    copy: bool,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    let profile_name = match profile_name {
        Some(name) => name,
        None => select_profile(&profile_manager, quiet)?,
    };

    let profile = profile_manager.get_profile(&profile_name)?.clone();

    let oauth_client = OAuthClient::new(profile.clone()).await?;
    let auth_request = oauth_client.create_authorization_request()?;

    if !quiet {
        println!("Initiating OAuth 2.0 authorization flow...");
    }

    open_browser_with_fallback(&auth_request.authorization_url, quiet)?;

    let (code, state, server_opt) = if is_localhost_redirect_uri(&profile.redirect_uri) {
        // Use callback server for localhost URLs
        let port = port
            .or_else(|| extract_port_from_redirect_uri(&profile.redirect_uri))
            .unwrap_or(8080);

        let mut server = CallbackServer::new(port, &profile.redirect_uri)?;

        if verbose {
            println!("Starting callback server on port {port}");
        }

        let mut receiver = server.start().await?;

        if !quiet {
            println!("Waiting for authentication callback...");
            println!("Press Ctrl+C to cancel");
        }

        let callback_result = timeout(Duration::from_secs(300), receiver.recv())
            .await
            .map_err(|_| OidcError::Auth("Authentication timeout (5 minutes)".to_string()))?
            .ok_or_else(|| OidcError::Auth("Failed to receive callback".to_string()))?;

        if let Some(error) = callback_result.error {
            return Err(OidcError::Auth(format!(
                "Authentication failed: {} - {}",
                error,
                callback_result.error_description.unwrap_or_default()
            )));
        }

        (callback_result.code, callback_result.state, Some(server))
    } else {
        let code = handle_manual_code_entry(quiet).await?;
        (code, auth_request.state.clone(), None)
    };

    if let Some(server) = server_opt {
        // Exchange tokens in background while browser shows success page
        let server_clone = server.clone();
        let oauth_client_clone = oauth_client.clone();
        let code_clone = code.clone();
        let state_clone = state.clone();
        let auth_state_clone = auth_request.state.clone();
        let verifier_clone = auth_request.pkce_challenge.verifier.clone();

        tokio::spawn(async move {
            if verbose {
                println!("Received authorization code, exchanging for tokens...");
            }

            match oauth_client_clone
                .exchange_code_for_tokens(
                    &code_clone,
                    &state_clone,
                    &auth_state_clone,
                    &verifier_clone,
                )
                .await
            {
                Ok(token_response) => {
                    // Display tokens in terminal
                    if quiet {
                        println!("{}", serde_json::to_string(&token_response).unwrap());
                    } else {
                        display_tokens(&token_response, copy).unwrap_or_else(|e| {
                            eprintln!("Error displaying tokens: {e}");
                        });
                    }

                    // Set token on server so browser can access it
                    server_clone
                        .set_token(token_response.access_token.clone())
                        .await;

                    if !quiet {
                        println!();
                        println!("Token is now available in the browser.");
                    }
                }
                Err(e) => {
                    eprintln!("Error exchanging code for tokens: {e}");
                }
            }
        });

        if !quiet {
            println!("Authentication successful! Check your browser for the access token.");
            println!("Tokens will be displayed in the browser once ready...");
        }

        // Wait a shorter time for token to be available, then exit
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    } else {
        // For non-localhost redirect URIs, exchange tokens normally
        if verbose {
            println!("Received authorization code, exchanging for tokens...");
        }

        let token_response = oauth_client
            .exchange_code_for_tokens(
                &code,
                &state,
                &auth_request.state,
                &auth_request.pkce_challenge.verifier,
            )
            .await?;

        if quiet {
            println!("{}", serde_json::to_string(&token_response).unwrap());
        } else {
            display_tokens(&token_response, copy)?;
        }
    }

    Ok(())
}

