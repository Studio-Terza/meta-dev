mod workspace;

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use file_ports::Files;
use workspace_metadata::Metadata;

use crate::cli::command::Command;
use crate::cli::failure::Failure;
use workspace::Workspace;

#[derive(Debug)]
pub enum Request {
    Affected(Vec<PathBuf>),
    Widened(Vec<PathBuf>),
    History(Vec<PathBuf>),
    Sources(Vec<OsString>),
    Beyond { produced: PathBuf, changed: Vec<PathBuf> },
}

impl Request {
    pub fn parsed(command: Command, mut arguments: impl Iterator<Item = OsString>) -> Result<Self, Failure> {
        match command {
            Command::Affected => Ok(Self::Affected(arguments.map(PathBuf::from).collect())),
            Command::Widened => Ok(Self::Widened(arguments.map(PathBuf::from).collect())),
            Command::History => Ok(Self::History(arguments.map(PathBuf::from).collect())),
            Command::Sources => Ok(Self::Sources(arguments.collect())),
            Command::Beyond => arguments
                .next()
                .ok_or(Failure::Usage)
                .map(PathBuf::from)
                .map(|produced| Self::Beyond { produced, changed: arguments.map(PathBuf::from).collect() }),
        }
    }

    pub fn written(self, files: &impl Files, metadata: &Path, out: &mut impl Write) -> Result<(), Failure> {
        match self {
            Self::Affected(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).reached(&changed))?)
            }
            Self::Widened(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).reached(&changed).widened())?)
            }
            Self::History(changed) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).history(&changed))?)
            }
            Self::Sources(roots) => {
                Ok(write!(out, "{}", Workspace::from(&Self::metadata(files, metadata)?).sources(&roots))?)
            }
            Self::Beyond { produced, changed } => Ok(Workspace::from(&Self::metadata(files, metadata)?)
                .beyond(&produced, &changed)
                .map_or(Ok(()), |path| writeln!(out, "{}", path.display()))?),
        }
    }

    fn metadata(files: &impl Files, metadata: &Path) -> Result<Metadata, Failure> {
        let text = files
            .read_text(metadata)
            .map_err(|failure| Failure::Read { path: metadata.to_path_buf(), source: io::Error::from(failure) })?;
        Ok(serde_json::from_str::<Metadata>(text.as_ref())?.checked()?)
    }
}
