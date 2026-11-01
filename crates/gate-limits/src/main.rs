use std::process::ExitCode;

use gate_limits::Cli;

fn main() -> ExitCode {
    Cli::from_args(std::env::args_os().skip(1)).run()
}
