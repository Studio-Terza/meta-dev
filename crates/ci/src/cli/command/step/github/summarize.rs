use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use text_lines::LossyText;

use super::super::journal::Journal;
use super::Step;
use super::show::Show;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct Summarize {
    report: Journal,
    summary: Journal,
}

impl Step for Summarize {
    fn from_arguments(arguments: &[OsString]) -> Result<Summarize, Failure> {
        match arguments {
            [report, summary] => {
                Ok(Summarize { report: Journal::file(Path::new(report)), summary: Journal::file(Path::new(summary)) })
            }
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, _out: &mut impl Write) -> Result<Exit, Failure> {
        let mut shown = Vec::new();
        Show::shown(&self.report, &mut shown)?;
        self.summary.append(LossyText::from(shown.as_slice())).map(|()| Exit::Success)
    }
}
