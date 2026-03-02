use claude_switch::cli::parser::{parse_args, CliMode};
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
            eprintln!("Interactive mode coming soon. Use --<profile> to switch.");
            Ok(1)
        }
        CliMode::Direct(args) => run_switch(args),
    }
}
