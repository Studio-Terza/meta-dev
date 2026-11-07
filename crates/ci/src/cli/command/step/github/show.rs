use std::ffi::OsString;
use std::io::Write;
use std::iter;
use std::path::PathBuf;

use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports, Text};

#[derive(Debug)]
pub struct Show {
    journals: Vec<Journal>,
}

impl Show {
    const ABSENT: &'static str = "not produced\n";

    pub(super) fn shown<'journal>(
        files: &impl Files,
        journals: &'journal [Journal],
        out: &mut impl Write,
    ) -> Result<(), Failure<'journal>> {
        journals.iter().zip(Journal::read_present(files, journals.iter().collect())).try_for_each(|(journal, text)| {
            writeln!(out, "{journal}:")?;
            let text = text.unwrap_or_else(|| Ok(Text::new(Self::ABSENT.to_owned())))?;
            Ok(write!(out, "{text}")?)
        })
    }
}

impl Step for Show {
    fn from_arguments(arguments: Vec<OsString>) -> Result<Self, Failure<'static>> {
        let mut arguments = Arguments::from(arguments);
        let first = arguments.taken()?;
        Ok(Self {
            journals: iter::once(first)
                .chain(arguments.rest())
                .map(|file| Journal::from(PathBuf::from(file)))
                .collect(),
        })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        Self::shown(ports.files(), &self.journals, out).map(|()| Exit::Success)
    }
}
