use console_ports::Process;
use design::{Agents, Cli, Failure, Outcome};
use design_store::StreamSchema;
use dispatch_ports::{HeadlessClaude, SystemClock};
use exit::Exit;
use file_ports::Disk;
use store_ports::Sqlite;

fn main() -> Exit<Outcome, Failure> {
    Exit::from(Process::answer(|arguments, out, _err| {
        Cli::new(&Process, Disk, Sqlite::file::<StreamSchema>, Agents::new(HeadlessClaude::from, SystemClock))
            .rooted(arguments.into_iter(), out)
    }))
}
