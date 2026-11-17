use crate::cli::command::Telling;
use console_ports::Shown;
use std::ffi::OsString;
use std::path::Path;

use super::Step;
use super::{Journal, Measured, Revision};
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct MeasuredHistory {
    measured: Measured,
    file: Journal,
    revision: Revision,
}

impl Step for MeasuredHistory {
    fn from_arguments(given: Vec<OsString>) -> Result<MeasuredHistory, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let measured = &arguments.taken()?;
        let file = &arguments.taken()?;
        let revision = &arguments.taken()?;
        arguments.ended()?;
        Ok(MeasuredHistory {
            measured: Measured::file(Path::new(measured)),
            file: Journal::file(Path::new(file)),
            revision: Revision::new(revision.to_str().ok_or(Failure::Usage)?.to_owned()),
        })
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
