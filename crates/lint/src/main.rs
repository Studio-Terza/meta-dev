use std::process::ExitCode;

use console_ports::{Process, Toolchain};
use file_ports::Disk;
use lint_runner::Cli;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| Cli::from_args(arguments.into_iter()).report_to(&Disk, &Toolchain, out, err))
}
