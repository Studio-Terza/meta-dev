use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::slice;

use text_lines::LossyText;

use super::super::journal::Journal;
use super::Step;
use super::show::Show;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct Summarize {
    report: Journal,
    summary: Journal,
}

impl Step for Summarize {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let report = &arguments.taken()?;
        let summary = &arguments.taken()?;
        arguments.ended()?;
        Ok(Self { report: Journal::file(Path::new(report)), summary: Journal::file(Path::new(summary)) })
    }

    fn carry_out(&self, ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let mut shown = Vec::new();
        Show::shown(ports.files(), slice::from_ref(&self.report), &mut shown)?;
        self.summary.append(ports.files(), LossyText::from(shown.as_slice())).map(|()| Exit::Success)
    }
}
