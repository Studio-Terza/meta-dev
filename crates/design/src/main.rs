#![deny(clippy::wildcard_enum_match_arm)]

mod ports;

use std::process::ExitCode;

use console_ports::{Answer, Process, Shown};
use design::{Agents, Cli};
use dispatch_ports::SystemClock;
use file_ports::Disk;
use ports::SystemShell;
use sqlite_ports::SqliteOpener;

fn main() -> ExitCode {
    Process::tell(|arguments| {
        let uttered =
            Cli::new(&Process, Disk, SqliteOpener, Agents::new(SystemClock, SystemShell)).rooted(arguments.into_iter());
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        Answer::reported(uttered.ended()).showing(shown)
    })
}
