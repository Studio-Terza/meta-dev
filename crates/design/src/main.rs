#![deny(clippy::wildcard_enum_match_arm)]

use std::process::ExitCode;

use console_ports::{Answer, Process, Shown};
use design::{Agents, Cli};
use dispatch_ports::{Agent, HeadlessClaude, SystemClock, SystemShell};
use file_ports::Disk;
use sqlite_ports::SqliteOpener;

fn main() -> ExitCode {
    Process::tell(|arguments| {
        let uttered = Cli::new(
            &Process,
            Disk,
            SqliteOpener,
            Agents::new(|scratch| HeadlessClaude::new(scratch, Agent::claude()), SystemClock, SystemShell),
        )
        .rooted(arguments.into_iter());
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        Answer::reported(uttered.ended()).showing(shown)
    })
}
