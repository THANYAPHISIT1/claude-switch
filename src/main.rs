use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command};

// Service name in Keychain used by Claude Code (may need to change if Anthropic updates the name)
const KEYCHAIN_SERVICE: &str = "claude-code"; 

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: claude-switch --<account_name> [additional flags]");
        exit(1);
    }

    let first_arg = &args[1];
    if !first_arg.starts_with("--") {
        eprintln!("❌ Error: First argument must be the profile (e.g., --work)");
        exit(1);
    }
    
    let account = first_arg.trim_start_matches("--").to_string();
    let claude_args: Vec<String> = args.into_iter().skip(2).collect();

    let home_dir = env::var("HOME").expect("❌ Error: HOME environment variable not set.");
    
    // Set up path
    let mut config_dir = PathBuf::from(&home_dir);
    config_dir.push(format!(".claude-{}", account));
    
    let mut token_file = config_dir.clone();
    token_file.push("keychain_token.txt");

    // Create profile folder if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    println!("🚀 Profile: {}", account);

    // ==========================================
    // 1. RESTORE: Load token from file and put it in Keychain before running
    // ==========================================
    if token_file.exists() {
        if let Ok(saved_token) = fs::read_to_string(&token_file) {
            let saved_token = saved_token.trim();
            // Delete old entry first (prevent duplicate data error)
            let _ = Command::new("security")
                .args(["delete-generic-password", "-s", KEYCHAIN_SERVICE])
                .output();
            
            // Insert this account's entry
            let add_status = Command::new("security")
                .args(["add-generic-password", "-a", &env::var("USER").unwrap_or_default(), "-s", KEYCHAIN_SERVICE, "-w", saved_token])
                .status();
                
            if add_status.is_ok() {
                println!("🔓 Restored token for profile: {}", account);
            }
        }
    } else {
        // If no file exists (first run), clear keychain to force Claude to request new login
        let _ = Command::new("security")
            .args(["delete-generic-password", "-s", KEYCHAIN_SERVICE])
            .output();
        println!("✨ First time using this profile. You may need to login.");
    }

    // ==========================================
    // 2. RUN: Run Claude Code
    // ==========================================
    let mut child = Command::new("claude")
        .env("CLAUDE_CONFIG_DIR", &config_dir)
        .args(&claude_args)
        .spawn() // Use spawn() instead of status() for better manage connection Process 
        .expect("❌ Failed to execute 'claude' command");

    // Wait until /exit or Ctrl+C
    let exit_status = child.wait().expect("Failed to wait on child process");

    // ==========================================
    // 3. BACKUP: Save Token from Keychain into file for next time
    // ==========================================
    let find_output = Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()
        .expect("Failed to read from keychain");

    if find_output.status.success() {
        // convert Binary to String
        if let Ok(current_token) = String::from_utf8(find_output.stdout) {
            let current_token = current_token.trim();
            if !current_token.is_empty() {
                // write token into file for next time
                fs::write(&token_file, current_token)
                    .expect("Failed to save token to file");
                println!("💾 Token safely backed up for next time.");
            }
        }
    }

    // End process
    exit(exit_status.code().unwrap_or(1));
}