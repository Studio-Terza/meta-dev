use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use super::changed::{Changed, Question};
use super::journal::Journal;
use super::revision::Revision;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Utf8};
use crate::cli::ports::{Exit, Ports};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct History {
    #[field(Journaled)]
    metadata: Journal,
    #[field(Journaled)]
    file: Journal,
    #[field(Utf8, via = Revision::new)]
    revision: Revision,
}

impl Step for History {
    fn from_arguments(given: Vec<OsString>) -> Result<History, Failure<'static>> {
        History::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let record = Changed::answered(Question::History, &self.metadata, ports)?;
        let said = vec![Shown::said(format_args!("{}\t{record}", self.revision))];
        Ok(Telling::reported(
            said,
            self.file.append(ports.files(), format_args!("{}\t{record}", self.revision)).map(|()| Exit::Success),
        ))
    }
}
