use std::process::Command;

use crate::security::token::Profile;

pub fn run_claude(profile: &Profile, args: &[String]) -> Result<i32, String> {
    let mut command = Command::new("claude");

    if profile.is_default {
        // The default profile IS Claude Code's own `~/.claude`. Leave CLAUDE_CONFIG_DIR
        // unset so the child behaves exactly like running `claude` bare — setting it
        // explicitly would derive a different keychain entry, and any value inherited
        // from the parent shell would silently override the profile.
        command.env_remove("CLAUDE_CONFIG_DIR");
    } else {
        command.env("CLAUDE_CONFIG_DIR", &profile.config_dir);
    }

    let mut child = command
        .args(args)
        .spawn()
        .map_err(|_| "❌ Failed to execute 'claude' command".to_string())?;

    let exit_status = child
        .wait()
        .map_err(|e| format!("Failed to wait on child process: {}", e))?;

    Ok(exit_status.code().unwrap_or(1))
}
