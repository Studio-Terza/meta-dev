use std::fmt::Display;
use std::process::ExitCode;

use super::Severity;

#[derive(Debug)]
pub struct Stderr;

impl Stderr {
    const BLOCKING: u8 = 2;

    pub fn failed(failure: &impl Display, severity: Severity) -> ExitCode {
        eprintln!("{failure}");
        Stderr::code(severity)
    }

    fn code(severity: Severity) -> ExitCode {
        match severity {
            Severity::Failed => ExitCode::FAILURE,
            Severity::Blocking => ExitCode::from(Stderr::BLOCKING),
        }
    }
}
