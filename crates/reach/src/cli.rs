mod failure;
mod workspace;

use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use file_ports::Files;
use lint_reader::SourceFiles;

use failure::Failure;
use workspace::{Metadata, Workspace};

#[derive(Debug)]
pub struct Cli;

impl Cli {
    const AFFECTED: &'static str = "affected";
    const WIDENED: &'static str = "widened";
    const FAN_IN: &'static str = "fan-in";
    const HUBS: &'static str = "hubs";
    const HISTORY: &'static str = "history";
    const SOURCES: &'static str = "sources";
    const BEYOND: &'static str = "beyond";

    #[must_use]
    pub fn answer(
        arguments: impl Iterator<Item = OsString>,
        files: &impl Files,
        out: &mut impl Write,
        err: &mut impl Write,
    ) -> ExitCode {
        match Self::report(arguments, files, out) {
            Ok(()) => ExitCode::SUCCESS,
            Err(failure) => {
                writeln!(err, "{failure}").ok();
                ExitCode::FAILURE
            }
        }
    }

    fn report(
        mut arguments: impl Iterator<Item = OsString>,
        files: &impl Files,
        out: &mut impl Write,
    ) -> Result<(), Failure> {
        let command = arguments.next();
        let metadata = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
        match command.as_deref().and_then(OsStr::to_str) {
            Some(Self::AFFECTED) => {
                let changed: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, &metadata)?).reached(&changed))?)
            }
            Some(Self::WIDENED) => {
                let changed: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, &metadata)?).reached(&changed).widened())?)
            }
            Some(Self::HISTORY) => {
                let changed: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, &metadata)?).history(&changed))?)
            }
            Some(Self::SOURCES) => {
                let roots: Vec<OsString> = arguments.collect();
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, &metadata)?).sources(&roots))?)
            }
            Some(Self::BEYOND) => {
                let produced = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
                let changed: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
                let beyond = Workspace::from(&Self::metadata(files, &metadata)?).beyond(&produced, &changed);
                Ok(beyond.map_or(Ok(()), |path| writeln!(out, "{}", path.display()))?)
            }
            Some(Self::HUBS) => {
                let repository = arguments.next().ok_or(Failure::Usage).map(PathBuf::from)?;
                let readout = SourceFiles::read(files, &repository)?;
                Ok(write!(
                    out,
                    "{}",
                    Workspace::from(&Self::metadata(files, &metadata)?).hubs(readout.files(), &repository)
                )?)
            }
            Some(Self::FAN_IN) => Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, &metadata)?).fan_in())?),
            _ => Err(Failure::Usage),
        }
    }

    fn metadata(files: &impl Files, metadata: &Path) -> Result<Metadata, Failure> {
        let text = files
            .read_text(metadata)
            .map_err(|failure| Failure::Read { path: metadata.to_path_buf(), source: io::Error::from(failure) })?;
        serde_json::from_str::<Metadata>(text.as_ref())?.checked()
    }
}
