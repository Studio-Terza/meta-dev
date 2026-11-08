mod ports;

use std::fmt::Display;
use std::io;
use std::process::{ExitCode, Termination};

use ports::Stderr;

#[derive(Debug)]
pub struct Exit<O, F>(Result<O, F>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Failed,
    Blocking,
}

pub trait Severe {
    fn severity(&self) -> Severity;
}

impl<O, F> From<Result<O, F>> for Exit<O, F> {
    fn from(result: Result<O, F>) -> Exit<O, F> {
        Exit(result)
    }
}

impl<O, F: Display + Severe> Termination for Exit<O, F> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(_outcome) => ExitCode::SUCCESS,
            Err(failure) => Stderr::failed(&failure, failure.severity()),
        }
    }
}

impl Severe for io::Error {
    fn severity(&self) -> Severity {
        Severity::Failed
    }
}
