use std::process::ExitCode;

use console_ports::Process;
use design::{Cli, Streams};
use file_ports::Disk;
use port_core::Rooting;

fn main() -> ExitCode {
    Process::answer(|arguments, out, err| {
        Cli::new(&Process, Disk).rooted(Rooting::Shared, arguments.into_iter(), &mut Streams::new(out, err))
    })
}
