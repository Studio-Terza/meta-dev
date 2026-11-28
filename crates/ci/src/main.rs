use std::process::ExitCode;

use ci::Cli;
use console_ports::Process;
use dispatch_ports::SystemShell;
use file_ports::Disk;
use timer_ports::SystemClock;

fn main() -> ExitCode {
    Process::tell(|arguments| Cli::from_args(&arguments).answer(&Disk, SystemClock, SystemShell))
}
