use console_ports::Process;
use design::{Cli, Failure, Outcome};
use design_store::StreamSchema;
use exit::Exit;
use file_ports::Disk;
use store_ports::Sqlite;

fn main() -> Exit<Outcome, Failure> {
    Exit::from(Process::answer(|arguments, out, _err| {
        Cli::new(&Process, Disk, Sqlite::file::<StreamSchema>).rooted(arguments.into_iter(), out)
    }))
}
