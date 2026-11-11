#![deny(clippy::wildcard_enum_match_arm)]

use console_ports::Process;
use design::{Agents, Cli};
use dispatch_ports::{Agent, HeadlessClaude, SystemClock};
use exit::Exit;
use file_ports::Disk;
use sqlite_ports::SqliteOpener;

fn main() -> Exit {
    Process::answer(|arguments, out, err| {
        let answered = Cli::new(
            &Process,
            Disk,
            SqliteOpener,
            Agents::new(|scratch| HeadlessClaude::new(scratch, Agent::CLAUDE), SystemClock),
        )
        .rooted(arguments.into_iter(), out);
        Exit::reported(answered, err)
    })
}
