use std::io;
use std::process::ExitCode;

use ci::Cli;
use ci::ports::System;

fn main() -> ExitCode {
    Cli::answer(std::env::args_os().skip(1), &System, &mut io::stdout(), &mut io::stderr())
}
