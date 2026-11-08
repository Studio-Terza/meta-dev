use std::fmt::Display;
use std::process::ExitCode;

#[derive(Debug)]
pub struct Stderr;

impl Stderr {
    pub fn failed(failure: &impl Display) -> ExitCode {
        eprintln!("{failure}");
        ExitCode::FAILURE
    }
}
