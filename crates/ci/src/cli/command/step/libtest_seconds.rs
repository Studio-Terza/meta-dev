use crate::cli::command::Telling;
mod boundary;

use std::ffi::OsString;

use boundary::Finished;
use text_lines::Lines;

use super::Step;
use super::journal::Journal;
use super::measured::Measured;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Recording, Utf8};
use crate::cli::ports::{Exit, Ports};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct LibtestSeconds {
    #[field(Recording)]
    measured: Measured,
    #[field(Utf8, via = Stage::new)]
    stage: Stage,
    #[field(Journaled)]
    output: Journal,
}

impl Step for LibtestSeconds {
    fn from_arguments(given: Vec<OsString>) -> Result<LibtestSeconds, Failure<'static>> {
        LibtestSeconds::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let text = self.output.read(ports.files())?;
        let results: Vec<Finished> = text
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter_map(|line| Finished::read(self.output.as_ref(), &line))
            .collect::<Result<_, _>>()?;
        let total: Seconds = results.iter().copied().map(Finished::seconds).sum();
        Ok(Telling::ended(
            results
                .first()
                .map(|_| total)
                .ok_or_else(|| Failure::NoTestResult { path: self.output.as_ref() })
                .and_then(|summed| self.measured.append(ports.files(), &self.stage, summed))
                .map(|()| Exit::Success),
        ))
    }
}
