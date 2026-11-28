#![deny(clippy::wildcard_enum_match_arm)]

use std::path::Path;
use std::process::ExitCode;

use console_ports::{Answer, Process, Shown};
use design::{Agents, Cli, Lint};
use dispatch_ports::{SystemClock, SystemShell};
use file_ports::Disk;
use sqlite_ports::SqliteOpener;
use timer_ports::Monotonic;

fn main() -> ExitCode {
    Process::stream(|arguments, out, err| {
        if let Some(lint) = Lint::asked(&arguments) {
            ExitCode::from(lint.verdict(&Disk, Path::new(""), Monotonic::default(), out, err))
        } else {
            let uttered = Cli::new(&Process, Disk, SqliteOpener, Agents::new(SystemClock, SystemShell))
                .rooted(arguments.into_iter());
            let shown = AsRef::<Shown>::as_ref(&uttered).clone();
            Answer::reported(uttered.ended()).showing(shown).written(out, err)
        }
    })
}
