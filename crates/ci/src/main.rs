use std::process::ExitCode;

use ci::Cli;
use ci::ports::System;
use console_ports::Process;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| Cli::answer(arguments.into_iter(), &System, out, err))
}
