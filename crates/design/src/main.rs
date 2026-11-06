use console_ports::Process;
use design::{Cli, Failure, Outcome};
use exit::Exit;
use file_ports::Disk;

fn main() -> Exit<Outcome, Failure> {
    Exit::from(Process::answer(|arguments, out, _err| Cli::new(&Process, Disk).rooted(arguments.into_iter(), out)))
}
