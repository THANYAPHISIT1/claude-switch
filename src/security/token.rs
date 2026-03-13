use std::env;
use std::path::PathBuf;

pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
    pub token_file: PathBuf,
}

impl Profile {
    pub fn new(account: &str) -> Result<Self, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "❌ Error: Could not determine home directory.".to_string())?;

        let config_dir = home_dir.join(format!(".claude-{}", account));
        let token_file = config_dir.join("keychain_token.txt");

        Ok(Self {
            name: account.to_string(),
            config_dir,
            token_file,
        })
    }
}