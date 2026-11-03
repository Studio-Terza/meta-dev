use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use lint_runner::Cli;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| Cli::from_args(arguments.into_iter()).report_to(&Disk, out, err))
}
