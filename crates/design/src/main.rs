#![deny(clippy::wildcard_enum_match_arm)]

use console_ports::Process;
use design::{Agents, Cli, Failure, Outcome};
use design_store::StreamSchema;
use dispatch_ports::{Agent, Bell, HeadlessClaude, SystemClock};
use exit::Exit;
use file_ports::Disk;
use store_ports::Sqlite;

fn main() -> Exit<Outcome, Failure> {
    Exit::from(Process::answer(|arguments, out, _err| {
        let bell = Bell::default();
        Cli::new(
            &Process,
            Disk,
            Sqlite::file::<StreamSchema>,
            Agents::new(|scratch| HeadlessClaude::new(scratch, Agent::CLAUDE, &bell), SystemClock::new(&bell)),
        )
        .rooted(arguments.into_iter(), out)
    }))
}
