use std::io;
use std::process::ExitCode;

use mutation_report::Cli;

fn main() -> ExitCode {
    Cli::answer(std::env::args_os().skip(1), &mut io::stderr())
}
