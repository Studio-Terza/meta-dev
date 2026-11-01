use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use super::super::journal::Journal;
use super::Step;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct Show {
    files: Vec<PathBuf>,
}

impl Show {
    fn shown(file: &Journal, out: &mut impl Write) -> Result<(), Failure> {
        writeln!(out, "{file}:")?;
        Ok(write!(out, "{}", file.read_or("not produced\n")?)?)
    }
}

impl Step for Show {
    fn from_arguments(arguments: &[OsString]) -> Result<Show, Failure> {
        match arguments {
            [] => Err(Failure::Usage),
            files => Ok(Show { files: files.iter().map(PathBuf::from).collect() }),
        }
    }

    fn carry_out(&self, _ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        self.files.iter().try_for_each(|file| Self::shown(&Journal::file(file), out)).map(|()| Exit::Success)
    }
}
