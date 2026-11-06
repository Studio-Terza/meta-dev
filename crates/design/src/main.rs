use std::process::ExitCode;

use console_ports::Process;
use design::{Cli, Streams};
use file_ports::Disk;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| {
        Cli::new(&Process, Disk).rooted(arguments.into_iter(), &mut Streams::new(out, err))
    })
}
