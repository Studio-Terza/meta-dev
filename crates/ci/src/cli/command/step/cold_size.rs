mod kilobytes;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use file_ports::Files;
use kilobytes::Kilobytes;

use super::Step;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::Arguments;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct ColdSize {
    directory: PathBuf,
    out: Journal,
}

impl Step for ColdSize {
    fn from_arguments(given: Vec<OsString>) -> Result<ColdSize, Failure<'static>> {
        let mut arguments = Arguments::from(given);
        let directory = &arguments.taken()?;
        let out = &arguments.taken()?;
        arguments.ended()?;
        Ok(ColdSize { directory: PathBuf::from(directory), out: Journal::from(PathBuf::from(out)) })
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let size = ports
            .files()
            .bytes_below(&self.directory)
            .map_err(|failure| Failure::unusable(self.directory.as_path(), failure))?;
        let kilobytes = Kilobytes::from(size);
        write!(out, "{kilobytes}")?;
        self.out.emptied(ports.files())?.append(ports.files(), kilobytes).map(|()| Exit::Success)
    }
}
