use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use lint::{Cli, Verdict};
use timer_ports::Monotonic;

fn main() -> ExitCode {
    Process::stream(|arguments, out, err| {
        let verdict: Verdict = Cli::from_args(arguments).verdict(&Disk, Monotonic::default(), out, err);
        ExitCode::from(verdict)
    })
}
