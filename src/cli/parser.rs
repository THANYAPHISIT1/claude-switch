use std::env;

pub enum CliMode {
    Interactive,
    Direct(CliArgs),
    Version,
    Help,
    #[cfg(feature = "usage")]
    Usage(UsageArgs),
}

pub struct CliArgs {
    pub account: String,
    pub claude_args: Vec<String>,
}

#[cfg(feature = "usage")]
pub struct UsageArgs {
    pub profile: String,
}

pub fn parse_args() -> Result<CliMode, String> {
    let args: Vec<String> = env::args().collect();
    parse_args_from(&args)
}

fn parse_args_from(args: &[String]) -> Result<CliMode, String> {
    if args.len() < 2 {
        return Ok(CliMode::Interactive);
    }

    let first_arg = &args[1];

    // Handled before profile parsing: otherwise `--version` would be read as the
    // profile 'version' and silently create a ~/.claude-version directory.
    match first_arg.as_str() {
        "--version" | "-V" => return Ok(CliMode::Version),
        "--help" | "-h" => return Ok(CliMode::Help),
        _ => {}
    }

    #[cfg(feature = "usage")]
    if first_arg == "usage" {
        return parse_usage_args(args);
    }

    if !first_arg.starts_with("--") {
        return Err("❌ Error: First argument must be the profile (e.g., --work)".to_string());
    }

    let account = first_arg.trim_start_matches("--").to_string();
    let claude_args = args[2..].to_vec();

    Ok(CliMode::Direct(CliArgs { account, claude_args }))
}

#[cfg(feature = "usage")]
fn parse_usage_args(args: &[String]) -> Result<CliMode, String> {
    let profile = args
        .get(2)
        .cloned()
        .ok_or("❌ usage requires a profile: claude-switch usage <profile>")?;
    Ok(CliMode::Usage(UsageArgs { profile }))
}

pub fn print_version() {
    println!("claude-switch {}", env!("CARGO_PKG_VERSION"));
}

pub fn print_help() {
    println!("claude-switch — run Claude Code under isolated profiles\n");
    println!("USAGE");
    println!("  claude-switch                       interactive profile picker");
    println!("  claude-switch --<profile> [args…]   run claude with that profile");
    println!("  claude-switch --default [args…]     run claude with ~/.claude");
    #[cfg(feature = "usage")]
    println!("  claude-switch usage <profile>       show session usage");
    println!();
    println!("FLAGS");
    println!("  -h, --help      show this help");
    println!("  -V, --version   show version");
    println!();
    println!("PROFILES");
    println!("  --<name> maps to ~/.claude-<name>; --default maps to ~/.claude.");
    println!("  Trailing arguments are passed through to claude unchanged.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interactive_when_no_args() {
        let args = vec!["cs".to_string()];
        assert!(matches!(parse_args_from(&args), Ok(CliMode::Interactive)));
    }

    #[test]
    fn direct_parses_account_name() {
        let args = vec!["cs".to_string(), "--work".to_string()];
        let result = parse_args_from(&args).unwrap();
        if let CliMode::Direct(a) = result {
            assert_eq!(a.account, "work");
        } else {
            panic!("Expected Direct mode");
        }
    }

    #[test]
    fn passthrough_args_captured() {
        let args = vec!["cs".to_string(), "--work".to_string(), "--version".to_string()];
        let result = parse_args_from(&args).unwrap();
        if let CliMode::Direct(a) = result {
            assert_eq!(a.claude_args, vec!["--version"]);
        } else {
            panic!("Expected Direct mode");
        }
    }

    #[test]
    fn err_when_no_double_dash() {
        let args = vec!["cs".to_string(), "work".to_string()];
        assert!(parse_args_from(&args).is_err());
    }

    #[test]
    fn version_flag_is_not_a_profile() {
        for flag in ["--version", "-V"] {
            let args = vec!["cs".to_string(), flag.to_string()];
            assert!(
                matches!(parse_args_from(&args), Ok(CliMode::Version)),
                "{} should parse as Version",
                flag
            );
        }
    }

    #[test]
    fn help_flag_is_not_a_profile() {
        for flag in ["--help", "-h"] {
            let args = vec!["cs".to_string(), flag.to_string()];
            assert!(
                matches!(parse_args_from(&args), Ok(CliMode::Help)),
                "{} should parse as Help",
                flag
            );
        }
    }

    #[test]
    fn version_flag_wins_over_profile_parsing() {
        // Regression: `--version` used to become the profile 'version' and create
        // a ~/.claude-version directory before claude was even spawned.
        let args = vec!["cs".to_string(), "--version".to_string()];
        assert!(!matches!(parse_args_from(&args), Ok(CliMode::Direct(_))));
    }

    #[cfg(feature = "usage")]
    #[test]
    fn usage_parses_profile() {
        let args = vec!["cs".to_string(), "usage".to_string(), "work".to_string()];
        let result = parse_args_from(&args).unwrap();
        if let CliMode::Usage(a) = result {
            assert_eq!(a.profile, "work");
        } else {
            panic!("Expected Usage mode");
        }
    }

    #[cfg(feature = "usage")]
    #[test]
    fn err_usage_without_profile() {
        let args = vec!["cs".to_string(), "usage".to_string()];
        assert!(parse_args_from(&args).is_err());
    }

}
