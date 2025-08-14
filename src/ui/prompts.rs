use crate::error::{OidcError, Result};
use crate::profile::ProfileManager;
use std::io::{self, Write};

pub fn select_profile(profile_manager: &ProfileManager, quiet: bool) -> Result<String> {
    let profiles = profile_manager.list_profiles();

    if profiles.is_empty() {
        return Err(OidcError::Profile(
            "No profiles found. Create a profile first using 'create' command.".to_string(),
        ));
    }

    if profiles.len() == 1 {
        return Ok(profiles[0].clone());
    }

    if quiet {
        return Err(OidcError::Profile(
            "Multiple profiles available. Please specify a profile name.".to_string(),
        ));
    }

    println!("Multiple profiles available:");
    for (i, profile) in profiles.iter().enumerate() {
        println!("  {}. {}", i + 1, profile);
    }

    loop {
        print!("Select a profile (1-{}): ", profiles.len());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= profiles.len() {
                return Ok(profiles[choice - 1].clone());
            }
        }

        println!(
            "Invalid selection. Please enter a number between 1 and {}.",
            profiles.len()
        );
    }
}

pub fn prompt_input(prompt: &str, required: bool) -> Result<String> {
    loop {
        print!("{prompt}: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() && required {
            println!("This field is required. Please enter a value.");
            continue;
        }

        return Ok(input.to_string());
    }
}

pub fn prompt_input_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{prompt} [{default}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

pub fn prompt_input_with_current(prompt: &str, current: &str) -> Result<String> {
    print!("{prompt} [{current}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(current.to_string())
    } else {
        Ok(input.to_string())
    }
}

pub fn prompt_optional_input(prompt: &str) -> Result<Option<String>> {
    print!("{prompt}: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}

pub fn prompt_optional_input_with_current(
    prompt: &str,
    current: Option<&str>,
) -> Result<Option<String>> {
    let display_current = current.unwrap_or("none");
    print!("{prompt} [{display_current}]: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(current.map(|s| s.to_string()))
    } else if input == "none" || input == "null" {
        Ok(None)
    } else {
        Ok(Some(input.to_string()))
    }
}