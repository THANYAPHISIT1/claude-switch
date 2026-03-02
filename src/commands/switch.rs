use crate::cli::parser::CliArgs;
use crate::proxy::runner;
use crate::security;
use crate::utils::{fs, logger};

pub fn run_switch(args: CliArgs) -> Result<i32, String> {
    let profile = security::Profile::new(&args.account)?;
    fs::ensure_dir(&profile.config_dir)?;
    logger::info(&format!("Profile: {}", profile.name));

    let restored = security::restore(&profile)?;
    if restored {
        logger::success(&format!("Restored token for profile: {}", profile.name));
    } else {
        logger::warn("First time using this profile. You may need to login.");
    }

    let exit_code = runner::run_claude(&profile, &args.claude_args)?;

    if security::backup(&profile)? {
        logger::saved("Token safely backed up for next time.");
    }

    Ok(exit_code)
}
