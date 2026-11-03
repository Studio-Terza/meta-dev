mod command;
mod failure;
mod request;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use file_ports::Files;

use command::Command;
use failure::Failure;
use request::Request;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    #[must_use]
    pub fn answer(
        arguments: impl Iterator<Item = OsString>,
        files: &impl Files,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> ExitCode {
        Self::report(arguments, files, out).map_or_else(|failure| Self::refused(&failure, err), |()| ExitCode::SUCCESS)
    }

    fn report(
        mut arguments: impl Iterator<Item = OsString>,
        files: &impl Files,
        out: &mut impl Write,
    ) -> Result<(), Failure> {
        let command = arguments.next();
        let metadata = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
        command
            .as_deref()
            .and_then(OsStr::to_str)
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Command>)
            .and_then(|command| Request::parsed(command, arguments))
            .and_then(|request| request.written(files, &metadata, out))
    }

    fn refused(failure: &Failure, err: &mut impl Write) -> ExitCode {
        writeln!(err, "{failure}").ok();
        ExitCode::FAILURE
    }
}
