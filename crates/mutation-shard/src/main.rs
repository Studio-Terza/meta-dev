use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use mutation_shard::Cli;

fn main() -> ExitCode {
    Process::tell(|arguments| Cli::answer(arguments.into_iter(), &Disk))
}
