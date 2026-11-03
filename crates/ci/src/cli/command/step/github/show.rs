use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use file_ports::Files;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports, Text};

#[derive(Debug)]
pub struct Show {
    files: Vec<PathBuf>,
}

impl Show {
    const ABSENT: &'static str = "not produced\n";

    pub(super) fn shown(files: &impl Files, journals: &[Journal], out: &mut impl Write) -> Result<(), Failure> {
        journals.iter().zip(Journal::read_present(files, journals)).try_for_each(|(journal, text)| {
            writeln!(out, "{journal}:")?;
            let text = text.unwrap_or_else(|| Ok(Text::new(Self::ABSENT.to_owned())))?;
            Ok(write!(out, "{text}")?)
        })
    }
}

impl Step for Show {
    fn from_arguments(arguments: &[OsString]) -> Result<Show, Failure> {
        match arguments {
            [] => Err(Failure::Usage),
            files => Ok(Show { files: files.iter().map(PathBuf::from).collect() }),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let journals: Vec<Journal> = self.files.iter().map(|file| Journal::file(file)).collect();
        Self::shown(ports.files(), &journals, out).map(|()| Exit::Success)
    }
}
