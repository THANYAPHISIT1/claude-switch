use std::process::Command;

use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::cli::parser::CliArgs;
use crate::security::Profile;
use crate::utils::fs;

pub fn run_interactive_mode() -> Result<CliArgs, String> {
    loop {
        let profiles = fs::discover_profile_names()?;

        if profiles.is_empty() {
            eprintln!("No profiles found. Create your first one.");
            handle_new_profile()?;
            continue;
        }

        match show_main_picker(&profiles)? {
            MenuAction::Switch(account) => return Ok(CliArgs { account, claude_args: vec![] }),
            MenuAction::NewProfile => handle_new_profile()?,
            MenuAction::Manage => handle_manage(&profiles)?,
            MenuAction::Skip => {}
            MenuAction::Quit => std::process::exit(0),
        }
    }
}

enum MenuAction {
    Switch(String),
    NewProfile,
    Manage,
    Skip,
    Quit,
}

fn show_main_picker(profiles: &[String]) -> Result<MenuAction, String> {
    let home = dirs::home_dir().unwrap_or_default();

    let mut items: Vec<String> = profiles
        .iter()
        .map(|name| {
            let token_path = home.join(format!(".claude-{}/keychain_token.txt", name));
            let (indicator, status) = if token_path.exists() {
                ("●", "🔑 saved")
            } else {
                ("○", "✨ new ")
            };
            format!("  {} {:<15} [{:<10}]  ~/.claude-{}", indicator, name, status, name)
        })
        .collect();

    let separator_idx = items.len();
    items.push("  ─────────────────────────────────────────────".to_string());
    let new_profile_idx = items.len();
    items.push("  + New profile".to_string());
    let manage_idx = items.len();
    items.push("  ⚙  Manage profiles...".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("🔄 Claude-Switch — Profile Manager")
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .map_err(|e| e.to_string())?;

    Ok(match selection {
        None => MenuAction::Quit,
        Some(i) if i == separator_idx => MenuAction::Skip,
        Some(i) if i == new_profile_idx => MenuAction::NewProfile,
        Some(i) if i == manage_idx => MenuAction::Manage,
        Some(i) => MenuAction::Switch(profiles[i].clone()),
    })
}

fn handle_new_profile() -> Result<(), String> {
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("New profile name")
        .interact_text()
        .map_err(|e| e.to_string())?;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Ok(());
    }

    let profile = Profile::new(&name)?;
    fs::ensure_dir(&profile.config_dir)?;
    println!("✅ Created profile '{}' at {}", name, profile.config_dir.display());

    Ok(())
}

fn handle_manage(profiles: &[String]) -> Result<(), String> {
    let mut items: Vec<String> = profiles.iter().map(|n| format!("  {}", n)).collect();
    let back_idx = items.len();
    items.push("  ← Back".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("⚙  Manage — select a profile")
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .map_err(|e| e.to_string())?;

    let idx = match selection {
        None => return Ok(()),
        Some(i) => i,
    };

    if idx == back_idx {
        return Ok(());
    }

    manage_single_profile(&profiles[idx])
}

fn manage_single_profile(name: &str) -> Result<(), String> {
    let profile = Profile::new(name)?;

    let items = ["  🗑  Delete profile", "  📂  Reveal in Finder", "  ← Back"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("⚙  Manage: {}", name))
        .items(&items)
        .default(2)
        .interact_on_opt(&Term::stderr())
        .map_err(|e| e.to_string())?;

    match selection {
        Some(0) => delete_profile(&profile, name)?,
        Some(1) => reveal_in_finder(&profile)?,
        _ => {}
    }

    Ok(())
}

fn delete_profile(profile: &Profile, name: &str) -> Result<(), String> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Delete '{}' at {}? This cannot be undone.",
            name,
            profile.config_dir.display()
        ))
        .default(false)
        .interact()
        .map_err(|e| e.to_string())?;

    if confirmed {
        std::fs::remove_dir_all(&profile.config_dir)
            .map_err(|e| format!("Failed to delete profile: {}", e))?;
        println!("🗑  Deleted profile '{}'", name);
    }

    Ok(())
}

fn reveal_in_finder(profile: &Profile) -> Result<(), String> {
    Command::new("open")
        .arg(&profile.config_dir)
        .status()
        .map_err(|e| format!("Failed to open Finder: {}", e))?;
    Ok(())
}
