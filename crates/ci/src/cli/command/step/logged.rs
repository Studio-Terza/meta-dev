use crate::cli::command::Telling;
use std::ffi::OsString;
use std::path::PathBuf;

use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Location, Word};
use crate::cli::ports::{Invocation, Ports};
use grammar_output::Record;

#[derive(Debug)]
pub struct Logged {
    log: PathBuf,
    invocation: Invocation,
}

#[derive(Debug, Record)]
#[record(source = Arguments)]
struct LoggedWords {
    #[field(Location)]
    log: PathBuf,
    #[field(Word)]
    program: OsString,
    #[repeated(Word)]
    rest: Vec<OsString>,
}

impl From<LoggedWords> for Logged {
    fn from(words: LoggedWords) -> Logged {
        Logged { log: words.log, invocation: Invocation::new(words.program, words.rest) }
    }
}

impl Step for Logged {
    fn from_arguments(given: Vec<OsString>) -> Result<Logged, Failure<'static>> {
        LoggedWords::read(Arguments::from(given))?.ended().map(Logged::from)
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        Ok(Telling::ended(
            ports
                .logged(&self.invocation, &self.log)
                .map(|run| run.exit())
                .map_err(|source| Failure::Program { program: self.invocation.program().to_owned(), source }),
        ))
    }
}
