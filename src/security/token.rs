use std::env;
use std::path::PathBuf;

pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
    pub token_file: PathBuf,
}

impl Profile {
    pub fn new(account: &str) -> Result<Self, String> {
        let home_dir = env::var("HOME")
            .map_err(|_| "❌ Error: HOME environment variable not set.".to_string())?;

        let mut config_dir = PathBuf::from(&home_dir);
        config_dir.push(format!(".claude-{}", account));

        let mut token_file = config_dir.clone();
        token_file.push("keychain_token.txt");

        Ok(Self {
            name: account.to_string(),
            config_dir,
            token_file,
        })
    }
}
