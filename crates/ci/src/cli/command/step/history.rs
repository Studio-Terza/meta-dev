use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::Changed;
use super::journal::Journal;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct History {
    metadata: Journal,
    file: Journal,
    revision: String,
}

impl History {
    const COMMAND: &'static str = "history";
}

impl Step for History {
    fn from_arguments(arguments: &[OsString]) -> Result<History, Failure> {
        match arguments {
            [metadata, file, revision] => Ok(History {
                metadata: Journal::file(Path::new(metadata)),
                file: Journal::file(Path::new(file)),
                revision: revision.to_str().ok_or(Failure::Usage)?.to_owned(),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure> {
        let record = Changed::answered(Self::COMMAND, &self.metadata, ports)?;
        write!(out, "{}\t{record}", self.revision)?;
        self.file.append(format_args!("{}\t{record}", self.revision)).map(|()| Exit::Success)
    }
}
