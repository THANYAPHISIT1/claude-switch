use crate::cli::parser::CliArgs;
use crate::proxy::runner;
use crate::security;
use crate::utils::{fs, logger};

pub fn run_switch(args: CliArgs) -> Result<i32, String> {
    let profile = security::Profile::new(&args.account)?;
    fs::ensure_dir(&profile.config_dir)?;
    logger::info(&format!("Profile: {}", profile.name));

    let exit_code = runner::run_claude(&profile, &args.claude_args)?;

    Ok(exit_code)
}
