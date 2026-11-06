use console_ports::Process;
use design::{Cli, Failure};
use file_ports::Disk;

fn main() -> Result<(), Failure> {
    Process::answer(|arguments, out, _err| Cli::new(&Process, Disk).rooted(arguments.into_iter(), out)).map(drop)
}
