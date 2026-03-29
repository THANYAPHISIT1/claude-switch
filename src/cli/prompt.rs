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
            let config_dir = home.join(format!(".claude-{}", name));
            let has_session = std::fs::read_dir(&config_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
            let (indicator, status) = if has_session {
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

    #[cfg(feature = "usage")]
    {
        let setup = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Set up statusline session usage for this profile?")
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;
        if setup {
            setup_statusline(&name)?;
        }
    }

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

    let mut items: Vec<&str> = vec![
        "  🗑  Delete profile",
        "  📂  Reveal in Finder",
        "  🔗  Set alias",
        "  ✂️   Remove alias",
    ];
    #[cfg(feature = "usage")]
    let statusline_idx = items.len();
    #[cfg(feature = "usage")]
    items.push("  📊  Setup statusline");
    let back_idx = items.len();
    items.push("  ← Back");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("⚙  Manage: {}", name))
        .items(&items)
        .default(back_idx)
        .interact_on_opt(&Term::stderr())
        .map_err(|e| e.to_string())?;

    match selection {
        Some(0) => delete_profile(&profile, name)?,
        Some(1) => reveal_in_finder(&profile)?,
        Some(2) => set_alias(name)?,
        Some(3) => remove_alias(name)?,
        #[cfg(feature = "usage")]
        Some(i) if i == statusline_idx => setup_statusline(name)?,
        _ => {}
    }

    Ok(())
}

fn set_alias(profile_name: &str) -> Result<(), String> {
    let default_alias = format!("cs{}", profile_name);

    let alias_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Alias name (must start with 'cs')")
        .default(default_alias)
        .validate_with(|input: &String| -> Result<(), String> {
            if input.starts_with("cs") && input.len() > 2 {
                Ok(())
            } else {
                Err("Alias must start with 'cs' and have at least one character after (e.g., cswork)".to_string())
            }
        })
        .interact_text()
        .map_err(|e| e.to_string())?;

    let zshrc = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".zshrc");

    let alias_line = format!("alias {}='claude-switch --{}'", alias_name, profile_name);
    let fast_line = format!(
        "alias {}-fast='claude-switch --{} --dangerously-skip-permissions'",
        alias_name, profile_name
    );
    let block = format!("\n# claude-switch: {} profile\n{}\n{}\n", profile_name, alias_line, fast_line);

    // Check for existing alias block and ask to overwrite
    let existing = std::fs::read_to_string(&zshrc).unwrap_or_default();
    let marker = format!("# claude-switch: {} profile", profile_name);

    if existing.contains(&marker) {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Alias for '{}' already exists in ~/.zshrc. Overwrite?", profile_name))
            .default(true)
            .interact()
            .map_err(|e| e.to_string())?;

        if !overwrite {
            return Ok(());
        }

        let cleaned = remove_alias_block(&existing, &marker);
        std::fs::write(&zshrc, cleaned)
            .map_err(|e| format!("Failed to update ~/.zshrc: {}", e))?;
    }

    // Append new alias block
    let mut content = std::fs::read_to_string(&zshrc).unwrap_or_default();
    content.push_str(&block);
    std::fs::write(&zshrc, content)
        .map_err(|e| format!("Failed to write to ~/.zshrc: {}", e))?;

    println!("✅ Added to ~/.zshrc:");
    println!("   {}", alias_line);
    println!("   {}-fast  (--dangerously-skip-permissions)", alias_name);
    println!("   Run: source ~/.zshrc  to apply");

    Ok(())
}

/// Removes the claude-switch alias block for a profile from the given zshrc content.
/// Strips the marker comment line + the 2 alias lines that follow it.
fn remove_alias_block(content: &str, marker: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut skip_until = 0usize;
    let kept: Vec<&str> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            if line.contains(marker) {
                skip_until = i + 3; // marker + 2 alias lines
                None
            } else if i < skip_until {
                None
            } else {
                Some(*line)
            }
        })
        .collect();
    kept.join("\n").trim_end().to_string() + "\n"
}

fn remove_alias(profile_name: &str) -> Result<(), String> {
    let zshrc = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".zshrc");

    let existing = std::fs::read_to_string(&zshrc).unwrap_or_default();
    let marker = format!("# claude-switch: {} profile", profile_name);

    if !existing.contains(&marker) {
        println!("ℹ️  No alias found for '{}' in ~/.zshrc", profile_name);
        return Ok(());
    }

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Remove alias for '{}' from ~/.zshrc?", profile_name))
        .default(true)
        .interact()
        .map_err(|e| e.to_string())?;

    if !confirmed {
        return Ok(());
    }

    let cleaned = remove_alias_block(&existing, &marker);
    std::fs::write(&zshrc, cleaned)
        .map_err(|e| format!("Failed to update ~/.zshrc: {}", e))?;

    println!("✂️  Removed alias for '{}' from ~/.zshrc", profile_name);
    println!("   Run: source ~/.zshrc  to apply");

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

#[cfg(feature = "usage")]
fn setup_statusline(profile_name: &str) -> Result<(), String> {
    println!("\n📊 Statusline Setup for '{}'", profile_name);
    println!("   You'll need two cookies from claude.ai:");
    println!("   1. Open Chrome DevTools → Application → Cookies → https://claude.ai");
    println!("   2. Copy the values of 'sessionKey' and 'cf_clearance'");
    println!();

    let session_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("sessionKey")
        .validate_with(|input: &String| -> Result<(), String> {
            if input.starts_with("sk-") {
                Ok(())
            } else {
                Err("sessionKey must start with 'sk-'".to_string())
            }
        })
        .interact_text()
        .map_err(|e| e.to_string())?;

    let cf_clearance: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("cf_clearance")
        .interact_text()
        .map_err(|e| e.to_string())?;

    let profile = Profile::new(profile_name)?;
    let profile_dir = &profile.config_dir;

    let cookie_path = crate::commands::usage::cookie_file(profile_name);
    let cookie_json = serde_json::json!({
        "sessionKey": session_key,
        "cf_clearance": cf_clearance,
    });
    std::fs::write(&cookie_path, serde_json::to_string_pretty(&cookie_json)
            .map_err(|e| format!("Failed to serialize cookies: {}", e))?)
        .map_err(|e| format!("Failed to write cookies: {}", e))?;

    let settings_path = profile_dir.join("settings.json");
    let mut settings: serde_json::Value = std::fs::read_to_string(&settings_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let script_path = home.join(".claude").join("statusline-profile.sh");
    let command = format!("sh {} {}", script_path.display(), profile_name);
    settings["statusLine"] = serde_json::json!({
        "type": "command",
        "command": command,
    });

    std::fs::write(&settings_path, serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    let should_write = if script_path.exists() {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Update existing statusline script?")
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?
    } else {
        true
    };

    if should_write {
        std::fs::create_dir_all(script_path.parent().unwrap())
            .map_err(|e| format!("Failed to create ~/.claude: {}", e))?;
        std::fs::write(&script_path, include_str!("../../assets/statusline-profile.sh"))
            .map_err(|e| format!("Failed to write statusline script: {}", e))?;
        println!("   Wrote {}", script_path.display());
    }

    println!("✅ Statusline configured for '{}'", profile_name);
    println!("   Reload Claude Code to see the status bar.");

    Ok(())
}
