use std::process::ExitCode;

use lint_runner::Cli;

fn main() -> ExitCode {
    Cli::from_args(std::env::args_os().skip(1)).run()
}
