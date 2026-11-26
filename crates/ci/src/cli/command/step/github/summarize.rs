use crate::cli::command::Telling;
use std::ffi::OsString;
use std::slice;

use super::super::journal::Journal;
use super::Step;
use super::show::Show;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled};
use crate::cli::ports::{Exit, Ports};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct Summarize {
    #[field(Journaled)]
    report: Journal,
    #[field(Journaled)]
    summary: Journal,
}

impl Step for Summarize {
    fn from_arguments(given: Vec<OsString>) -> Result<Summarize, Failure<'static>> {
        Summarize::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let shown = Show::shown(ports.files(), slice::from_ref(&self.report))?;
        Ok(Telling::ended(self.summary.append(ports.files(), &shown).map(|()| Exit::Success)))
    }
}
