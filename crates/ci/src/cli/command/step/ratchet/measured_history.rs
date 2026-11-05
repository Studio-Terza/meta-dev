use std::ffi::OsString;
use std::io::Write;
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
    fn from_arguments(arguments: Vec<OsString>) -> Result<MeasuredHistory, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
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

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let row = self.measured.row(ports.files())?;
        writeln!(out, "{}\t{row}", self.revision)?;
        self.file.append(ports.files(), format_args!("{}\t{row}\n", self.revision)).map(|()| Exit::Succeeded)
    }
}
