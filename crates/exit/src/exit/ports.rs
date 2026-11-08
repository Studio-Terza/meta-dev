use std::fmt::Display;
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Debug)]
pub struct Stderr;

impl Stderr {
    pub fn failed(failure: &impl Display) -> ExitCode {
        writeln!(io::stderr(), "{failure}").ok();
        ExitCode::FAILURE
    }
}
