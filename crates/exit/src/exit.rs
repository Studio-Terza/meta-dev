mod ports;

use std::fmt::Display;
use std::process::{ExitCode, Termination};

use ports::Stderr;

#[derive(Debug)]
pub struct Exit<O, F>(Result<O, F>);

impl<O, F> From<Result<O, F>> for Exit<O, F> {
    fn from(result: Result<O, F>) -> Exit<O, F> {
        Exit(result)
    }
}

impl<O, F: Display> Termination for Exit<O, F> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(_outcome) => ExitCode::SUCCESS,
            Err(failure) => Stderr::failed(&failure),
        }
    }
}
