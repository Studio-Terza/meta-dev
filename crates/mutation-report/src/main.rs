use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use mutation_report::Cli;

fn main() -> ExitCode {
    Process::answer(|arguments| Cli::answer(&arguments, &Disk))
}
