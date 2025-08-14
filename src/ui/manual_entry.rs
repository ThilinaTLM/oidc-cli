use crate::error::Result;
use crate::utils::url::parse_query_params;
use std::io::{self, Write};
use url::Url;

pub async fn handle_manual_code_entry(quiet: bool) -> Result<String> {
    if !quiet {
        println!("Since your redirect URI is not localhost, you'll need to manually enter the authorization code.");
        println!("After authorizing in your browser, copy the full callback URL or just the 'code' parameter.");
    }

    loop {
        print!("Enter the authorization code or full callback URL: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            println!("Authorization code cannot be empty. Please try again.");
            continue;
        }

        if let Ok(url) = Url::parse(input) {
            if let Some(query) = url.query() {
                let params = parse_query_params(query);
                if let Some(code) = params.get("code") {
                    return Ok(code.clone());
                }
            }
        }

        // If not a URL, treat as direct code
        if !input.contains("://") {
            return Ok(input.to_string());
        }

        println!("Could not extract authorization code from the input. Please try again.");
    }
}
