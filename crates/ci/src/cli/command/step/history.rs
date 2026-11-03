use std::ffi::OsString;
use std::io::Write;
use std::path::Path;

use super::Step;
use super::changed::{Changed, Question};
use super::journal::Journal;
use super::revision::Revision;
use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Ports};

#[derive(Debug)]
pub struct History {
    metadata: Journal,
    file: Journal,
    revision: Revision,
}

impl Step for History {
    fn from_arguments(arguments: Vec<OsString>) -> Result<History, Failure<'static>> {
        match arguments.as_slice() {
            [metadata, file, revision] => Ok(History {
                metadata: Journal::file(Path::new(metadata)),
                file: Journal::file(Path::new(file)),
                revision: Revision::new(revision.to_str().ok_or(Failure::Usage)?.to_owned()),
            }),
            _ => Err(Failure::Usage),
        }
    }

    fn carry_out(&self, ports: &impl Ports, out: &mut impl Write) -> Result<Exit, Failure<'_>> {
        let record = Changed::answered(Question::History, &self.metadata, ports)?;
        write!(out, "{}\t{record}", self.revision)?;
        self.file.append(ports.files(), format_args!("{}\t{record}", self.revision)).map(|()| Exit::Success)
    }
}
