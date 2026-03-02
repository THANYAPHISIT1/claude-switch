use std::env;
use std::process::Command;

use crate::security::token::Profile;

const KEYCHAIN_SERVICE: &str = "claude-code";

/// Restores the saved token into the macOS keychain before launching Claude.
/// Returns `Ok(true)` if a token was restored, `Ok(false)` on first run (clears stale session).
pub fn restore(profile: &Profile) -> Result<bool, String> {
    if !profile.token_file.exists() {
        // First run: clear stale keychain entry to force Claude to request a new login
        let _ = Command::new("security")
            .args(["delete-generic-password", "-s", KEYCHAIN_SERVICE])
            .output();
        return Ok(false);
    }

    let saved_token = std::fs::read_to_string(&profile.token_file)
        .map_err(|e| format!("Failed to read token file: {}", e))?;
    let saved_token = saved_token.trim();

    // Delete old entry first to prevent duplicate data error
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", KEYCHAIN_SERVICE])
        .output();

    let user = env::var("USER").unwrap_or_default();
    Command::new("security")
        .args([
            "add-generic-password",
            "-a",
            &user,
            "-s",
            KEYCHAIN_SERVICE,
            "-w",
            saved_token,
        ])
        .status()
        .map_err(|e| format!("Failed to add token to keychain: {}", e))?;

    Ok(true)
}

/// Backs up the current keychain token to file after Claude exits.
/// Returns `Ok(true)` if backed up, `Ok(false)` if keychain is empty (silent — normal after logout).
pub fn backup(profile: &Profile) -> Result<bool, String> {
    let find_output = Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()
        .map_err(|e| format!("Failed to read from keychain: {}", e))?;

    if !find_output.status.success() {
        return Ok(false);
    }

    let current_token = String::from_utf8(find_output.stdout).unwrap_or_default();
    let current_token = current_token.trim();

    if current_token.is_empty() {
        return Ok(false);
    }

    std::fs::write(&profile.token_file, current_token)
        .map_err(|e| format!("Failed to save token to file: {}", e))?;

    Ok(true)
}
