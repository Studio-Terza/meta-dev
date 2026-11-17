use std::process::ExitCode;

use console_ports::{Answer, Process};

fn main() -> ExitCode {
    Process::answer(|_arguments| Answer::new(ExitCode::SUCCESS).showing("cargo::rerun-if-changed=migrations\n"))
}
