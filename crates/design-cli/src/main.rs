#![deny(clippy::wildcard_enum_match_arm)]

use console_ports::Process;
use design::{Agents, Cli, Failure, Outcome};
use dispatch_ports::{Agent, HeadlessClaude, SystemClock};
use exit::Exit;
use file_ports::Disk;
use sqlite_ports::SqliteOpener;

fn main() -> Exit<Outcome, Failure> {
    Exit::from(Process::answer(|arguments, out, _err| {
        Cli::new(
            &Process,
            Disk,
            SqliteOpener,
            Agents::new(|scratch| HeadlessClaude::new(scratch, Agent::CLAUDE), SystemClock),
        )
        .rooted(arguments.into_iter(), out)
    }))
}
