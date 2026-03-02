use std::env;
use std::fs;
use std::path::PathBuf;

pub fn ensure_dir(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    Ok(())
}

pub fn read_token_file(path: &PathBuf) -> Option<String> {
    if path.exists() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

pub fn write_token_file(path: &PathBuf, token: &str) -> Result<(), String> {
    fs::write(path, token).map_err(|e| format!("Failed to write token file: {}", e))
}

/// Scans the home directory for ~/.claude-* profile directories and returns profile names.
pub fn discover_profile_names() -> Result<Vec<String>, String> {
    let home_dir = env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    let home = PathBuf::from(home_dir);

    let entries = fs::read_dir(&home)
        .map_err(|e| format!("Failed to read home directory: {}", e))?;

    let mut profiles: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(".claude-") && entry.path().is_dir() {
                Some(name_str[".claude-".len()..].to_string())
            } else {
                None
            }
        })
        .collect();

    profiles.sort();
    Ok(profiles)
}
