use crate::error::{OidcError, Result};
use crate::profile::ProfileManager;

pub fn handle_list(profile_manager: ProfileManager, quiet: bool) -> Result<()> {
    let profiles = profile_manager.list_profiles();

    if profiles.is_empty() {
        if !quiet {
            println!("No profiles found.");
        }
        return Ok(());
    }

    if quiet {
        for profile in profiles {
            println!("{profile}");
        }
    } else {
        println!("Available profiles:");
        for profile in profiles {
            println!("  • {profile}");
        }
    }

    Ok(())
}

pub fn handle_export(
    profile_manager: ProfileManager,
    file: std::path::PathBuf,
    profiles: Vec<String>,
    quiet: bool,
) -> Result<()> {
    let profile_names = if profiles.is_empty() {
        None
    } else {
        for name in &profiles {
            profile_manager.get_profile(name)?;
        }
        Some(profiles)
    };

    profile_manager.export_profiles(&file, profile_names)?;

    if !quiet {
        println!("✓ Profiles exported to {file:?} successfully.");
    }

    Ok(())
}

pub fn handle_import(
    profile_manager: &mut ProfileManager,
    file: std::path::PathBuf,
    overwrite: bool,
    quiet: bool,
) -> Result<()> {
    if !file.exists() {
        return Err(OidcError::Profile(format!(
            "Import file not found: {file:?}"
        )));
    }

    let imported_names = profile_manager.import_profiles(&file, overwrite)?;

    if !quiet {
        println!(
            "✓ Imported {} profile(s) from {:?}:",
            imported_names.len(),
            file
        );
        for name in imported_names {
            println!("  • {name}");
        }
    }

    Ok(())
}