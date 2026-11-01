use std::io;
use std::process::ExitCode;

use reach::Cli;

fn main() -> ExitCode {
    Cli::answer(std::env::args_os().skip(1), &mut io::stdout(), &mut io::stderr())
}
