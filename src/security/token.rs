use std::path::PathBuf;

/// Reserved profile name that maps to Claude Code's own config dir (`~/.claude`)
/// instead of a `~/.claude-<name>` directory.
pub const DEFAULT_PROFILE: &str = "default";

pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
    pub is_default: bool,
}

impl Profile {
    pub fn new(account: &str) -> Result<Self, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "❌ Error: Could not determine home directory.".to_string())?;

        let is_default = account == DEFAULT_PROFILE;
        let config_dir = if is_default {
            home_dir.join(".claude")
        } else {
            home_dir.join(format!(".claude-{}", account))
        };

        Ok(Self {
            name: account.to_string(),
            config_dir,
            is_default,
        })
    }

    /// Home-relative path shown in the TUI, e.g. `~/.claude-work` or `~/.claude`.
    pub fn display_path(&self) -> String {
        if self.is_default {
            "~/.claude".to_string()
        } else {
            format!("~/.claude-{}", self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_profile_maps_to_suffixed_dir() {
        let profile = Profile::new("work").unwrap();
        assert!(!profile.is_default);
        assert!(profile.config_dir.ends_with(".claude-work"));
        assert_eq!(profile.display_path(), "~/.claude-work");
    }

    #[test]
    fn default_profile_maps_to_plain_claude_dir() {
        let profile = Profile::new(DEFAULT_PROFILE).unwrap();
        assert!(profile.is_default);
        assert!(profile.config_dir.ends_with(".claude"));
        assert_eq!(profile.display_path(), "~/.claude");
    }
}
