use std::process::ExitCode;

use ci::Cli;
use console_ports::Process;

fn main() -> ExitCode {
    Process::stream(|arguments, out, err| Cli::from_args(&arguments).exit(out, err))
}
