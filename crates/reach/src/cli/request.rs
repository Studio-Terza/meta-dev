mod workspace;

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use file_ports::Files;
use lint_reader::SourceFiles;

use crate::cli::command::Command;
use crate::cli::failure::Failure;
use workspace::{Metadata, Workspace};

#[derive(Debug)]
pub enum Request {
    Affected(Vec<PathBuf>),
    Widened(Vec<PathBuf>),
    History(Vec<PathBuf>),
    Sources(Vec<OsString>),
    Beyond { produced: PathBuf, changed: Vec<PathBuf> },
    Hubs(PathBuf),
    FanIn,
}

impl Request {
    pub fn parsed(command: Command, mut arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        match command {
            Command::Affected => Ok(Request::Affected(arguments.map(PathBuf::from).collect())),
            Command::Widened => Ok(Request::Widened(arguments.map(PathBuf::from).collect())),
            Command::History => Ok(Request::History(arguments.map(PathBuf::from).collect())),
            Command::Sources => Ok(Request::Sources(arguments.collect())),
            Command::Beyond => Self::next_path(&mut arguments)
                .map(|produced| Request::Beyond { produced, changed: arguments.map(PathBuf::from).collect() }),
            Command::Hubs => Self::next_path(&mut arguments).map(Request::Hubs),
            Command::FanIn => Ok(Request::FanIn),
        }
    }

    pub fn written(self, files: &impl Files, metadata: &Path, out: &mut impl Write) -> Result<(), Failure> {
        match self {
            Request::Affected(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).reached(&changed))?)
            }
            Request::Widened(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).reached(&changed).widened())?)
            }
            Request::History(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).history(&changed))?)
            }
            Request::Sources(roots) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).sources(&roots))?)
            }
            Request::Beyond { produced, changed } => Ok(Workspace::from(&Self::metadata(files, metadata)?)
                .beyond(&produced, &changed)
                .map_or(Ok(()), |path| writeln!(out, "{}", path.display()))?),
            Request::Hubs(repository) => Self::hubs(&repository, files, metadata, out),
            Request::FanIn => Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).fan_in())?),
        }
    }

    fn next_path(arguments: &mut impl Iterator<Item = OsString>) -> Result<PathBuf, Failure> {
        arguments.next().ok_or(Failure::Usage).map(PathBuf::from)
    }

    fn hubs(repository: &Path, files: &impl Files, metadata: &Path, out: &mut impl Write) -> Result<(), Failure> {
        let readout = SourceFiles::read(files, repository)?;
        Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).hubs(readout.files(), repository))?)
    }

    fn metadata(files: &impl Files, metadata: &Path) -> Result<Metadata, Failure> {
        let text = files
            .read_text(metadata)
            .map_err(|failure| Failure::Read { path: metadata.to_path_buf(), source: io::Error::from(failure) })?;
        serde_json::from_str::<Metadata>(text.as_ref())?.checked()
    }
}
