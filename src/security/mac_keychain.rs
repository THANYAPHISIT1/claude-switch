use std::env;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::security::token::Profile;

/// Computes the keychain service name Claude Code uses for a given config directory.
/// Formula: "Claude Code-credentials-" + sha256(config_dir_path)[0..8].
/// NOTE: This formula can change in the future if Claude Code changes their keychain naming.
fn keychain_service(config_dir: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(config_dir.to_string_lossy().as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    format!("Claude Code-credentials-{}", &hash[..8])
}

/// Restores the saved token into the macOS keychain before launching Claude.
/// Returns `Ok(true)` if a token was restored, `Ok(false)` on first run (clears stale session).
pub fn restore(profile: &Profile) -> Result<bool, String> {
    let service = keychain_service(&profile.config_dir);

    if !profile.token_file.exists() {
        // First run: no saved token yet — let Claude use existing session or prompt login.
        // Each profile has its own keychain entry (via SHA256), so no need to clear anything.
        return Ok(false);
    }

    let saved_token = std::fs::read_to_string(&profile.token_file)
        .map_err(|e| format!("Failed to read token file: {}", e))?;
    let saved_token = saved_token.trim();

    // Delete old entry first to prevent duplicate data error
    let _ = Command::new("security")
        .args(["delete-generic-password", "-s", &service])
        .output();

    let user = env::var("USER").unwrap_or_default();
    Command::new("security")
        .args([
            "add-generic-password",
            "-a",
            &user,
            "-s",
            &service,
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
    let service = keychain_service(&profile.config_dir);

    let find_output = Command::new("security")
        .args(["find-generic-password", "-s", &service, "-w"])
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
