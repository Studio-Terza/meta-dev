use std::path::Path;
use std::process::ExitCode;

use console_ports::Process;
use file_ports::Disk;
use lint::{Cli, Verdict};

fn main() -> ExitCode {
    Process::stream(|arguments, out, err| {
        let verdict: Verdict = Cli::from_args(arguments).verdict(&Disk, Path::new(""), out, err);
        ExitCode::from(verdict)
    })
}
