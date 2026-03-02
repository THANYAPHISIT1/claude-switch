use std::process::Command;

use crate::security::token::Profile;

pub fn run_claude(profile: &Profile, args: &[String]) -> Result<i32, String> {
    let mut child = Command::new("claude")
        .env("CLAUDE_CONFIG_DIR", &profile.config_dir)
        .args(args)
        .spawn()
        .map_err(|_| "❌ Failed to execute 'claude' command".to_string())?;

    let exit_status = child
        .wait()
        .map_err(|e| format!("Failed to wait on child process: {}", e))?;

    Ok(exit_status.code().unwrap_or(1))
}
