use std::process::ExitCode;

use ci::Cli;
use console_ports::Process;

fn main() -> ExitCode {
    Process::tell(|arguments| Cli::from_args(&arguments).answer())
}
