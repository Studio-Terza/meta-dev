mod boundary;

use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use boundary::Finished;
use grammar::test_summary::{Malformed, Summary};
use text_lines::Lines;

use super::Step;
use super::journal::Journal;
use super::measured::Measured;
use super::seconds::Seconds;
use super::stage::Stage;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct LibtestSeconds {
    measured: Measured,
    stage: Stage,
    output: Journal,
}

impl Step for LibtestSeconds {
    fn from_arguments(arguments: Vec<OsString>) -> Result<LibtestSeconds, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let measured = &arguments.taken()?;
        let stage = &arguments.taken()?;
        let output = &arguments.taken()?;
        arguments.ended()?;
        Ok(LibtestSeconds {
            measured: Measured::file(Path::new(measured)),
            stage: Stage::new(stage.to_str().ok_or(Failure::Usage)?.to_owned()),
            output: Journal::file(Path::new(output)),
        })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let text = self.output.read(ports.files())?;
        let read: Vec<Option<Summary>> = text
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .map(|line| Summary::read(&line))
            .collect::<Result<_, Malformed>>()
            .map_err(|source| Failure::MalformedTestResult { path: self.output.as_ref(), source })?;
        let results: Vec<Finished> = read.into_iter().flatten().map(Finished::from).collect();
        let total: Seconds = results.iter().copied().map(Finished::seconds).sum();
        results
            .first()
            .map(|_| total)
            .ok_or_else(|| Failure::NoTestResult { path: self.output.as_ref() })
            .and_then(|total| self.measured.append(ports.files(), &self.stage, total))
            .map(|()| Exit::Success)
    }
}
