mod failure;
mod workspace;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use failure::Failure;
use workspace::Workspace;

#[derive(Debug)]
pub struct Cli;

impl Cli {
    const AFFECTED: &'static str = "affected";
    const FAN_IN: &'static str = "fan-in";

    #[must_use]
    pub fn answer(arguments: impl Iterator<Item = OsString>, out: &mut impl Write, err: &mut impl Write) -> ExitCode {
        match Self::report(arguments, out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }

    fn report(mut arguments: impl Iterator<Item = OsString>, out: &mut impl Write) -> Result<(), Failure> {
        let command = arguments.next();
        let metadata = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
        match command.as_deref().and_then(OsStr::to_str) {
            Some(Self::AFFECTED) => {
                let changed: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
                Ok(write!(out, "{}", Self::workspace(&metadata)?.reached(&changed))?)
            }
            Some(Self::FAN_IN) => Ok(write!(out, "{}", Self::workspace(&metadata)?.fan_in())?),
            _ => Err(Failure::Usage),
        }
    }

    fn workspace(metadata: &Path) -> Result<Workspace, Failure> {
        let text =
            fs::read_to_string(metadata).map_err(|source| Failure::Read { path: metadata.to_path_buf(), source })?;
        Workspace::parse(&text)
    }
}
