#![deny(clippy::wildcard_enum_match_arm)]

use std::process::ExitCode;

use console_ports::{Answer, Process, Shown};
use design::{Agents, Cli};
use dispatch_ports::SystemShell;
use timer_ports::SystemClock;

fn main() -> ExitCode {
    Process::tell(|arguments| {
        let uttered = Cli::new(&Process).rooted(Agents::new(SystemClock, SystemShell), arguments.into_iter());
        let shown = AsRef::<Shown>::as_ref(&uttered).clone();
        Answer::reported(uttered.ended()).showing(shown)
    })
}
