use claude_switch::cli::parser::{parse_args, CliMode};
use claude_switch::cli::prompt;
use claude_switch::commands::switch::run_switch;

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> Result<i32, String> {
    match parse_args()? {
        CliMode::Interactive => {
            let args = prompt::run_interactive_mode()?;
            run_switch(args)
        }
        CliMode::Direct(args) => run_switch(args),
        #[cfg(feature = "usage")]
        CliMode::Usage(args) => {
            claude_switch::commands::usage::run_usage(&args.profile);
            Ok(0)
        }
    }
}
