use std::process::ExitCode;

use ci::Cli;
use ci::ports::System;
use console_ports::Process;

fn main() -> ExitCode {
    Process::tell(|arguments| Cli::parsed(arguments.into_iter(), &System))
}
