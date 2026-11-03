use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use reach::Cli;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| Cli::answer(arguments.into_iter(), &Disk, out, err))
}
