use std::env;

pub enum CliMode {
    Interactive,
    Direct(CliArgs),
}

pub struct CliArgs {
    pub account: String,
    pub claude_args: Vec<String>,
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
    if !first_arg.starts_with("--") {
        return Err("❌ Error: First argument must be the profile (e.g., --work)".to_string());
    }

    let account = first_arg.trim_start_matches("--").to_string();
    let claude_args = args[2..].to_vec();

    Ok(CliMode::Direct(CliArgs { account, claude_args }))
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
}
