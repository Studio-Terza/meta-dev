use crate::cli::command::Telling;
use console_ports::Shown;
mod kilobytes;

use std::ffi::OsString;
use std::path::PathBuf;

use file_ports::Files;
use kilobytes::Kilobytes;

use super::Step;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::command::step::{Arguments, Ended, Journaled, Location};
use crate::cli::ports::{Exit, Ports, Text};
use grammar_output::Record;

#[derive(Debug, Record)]
#[record(source = Arguments)]
pub(crate) struct ColdSize {
    #[field(Location)]
    directory: PathBuf,
    #[field(Journaled)]
    out: Journal,
}

impl Step for ColdSize {
    fn from_arguments(given: Vec<OsString>) -> Result<ColdSize, Failure<'static>> {
        ColdSize::read(Arguments::from(given))?.ended()
    }

    fn carry_out<'step>(&'step self, ports: &'step impl Ports) -> Result<Telling<'step>, Failure<'step>> {
        let size = ports
            .files()
            .bytes_below(&self.directory)
            .map_err(|failure| Failure::unusable(self.directory.as_path(), failure))?;
        let kilobytes = Kilobytes::from(size);
        let said = vec![Shown::said(format_args!("{kilobytes}"))];
        self.out.replace(ports.files(), &Text::default())?;
        Ok(Telling::reported(said, self.out.append(ports.files(), kilobytes).map(|()| Exit::Success)))
    }
}
