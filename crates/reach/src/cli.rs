mod command;
mod failure;
mod request;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::ExitCode;

use console_ports::Answer;
use file_ports::Files;

use command::Command;
use failure::Failure;
use request::Request;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    pub fn answer(arguments: impl Iterator<Item = OsString>, files: &impl Files) -> Answer {
        Self::report(arguments, files)
            .unwrap_or_else(|failure| Answer::complained(ExitCode::FAILURE, format_args!("{failure}\n")))
    }

    fn report(mut arguments: impl Iterator<Item = OsString>, files: &impl Files) -> Result<Answer, Failure> {
        let command = arguments.next();
        let metadata = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
        command
            .as_deref()
            .and_then(OsStr::to_str)
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Command>)
            .and_then(|parsed| Request::parsed(parsed, arguments))
            .and_then(|request| request.written(files, &metadata))
    }
}
