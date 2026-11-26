use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;

use super::Step;
use super::{Journal, Measured, Revision};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Recording, Utf8};
use crate::cli::ports::{Exit, Ports};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub struct MeasuredHistory {
    #[field(Recording)]
    measured: Measured,
    #[field(Journaled)]
    file: Journal,
    #[field(Utf8, via = Revision::new)]
    revision: Revision,
}

impl Step for MeasuredHistory {
    fn from_arguments(given: Vec<OsString>) -> Result<MeasuredHistory, Failure<'static>> {
        MeasuredHistory::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let row = self.measured.row(ports.files())?;
        let said = vec![Shown::said(format_args!("{}\t{row}\n", self.revision))];
        Ok(Telling::reported(
            said,
            self.file.append(ports.files(), format_args!("{}\t{row}\n", self.revision)).map(|()| Exit::Success),
        ))
    }
}
