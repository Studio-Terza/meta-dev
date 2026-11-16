mod workspace;

use std::ffi::OsString;
use std::io;
use std::process::ExitCode;

use console_ports::Answer;
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
    pub fn parsed(command: Command, mut arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        match command {
            Command::Affected => Ok(Request::Affected(arguments.map(PathBuf::from).collect())),
            Command::Widened => Ok(Request::Widened(arguments.map(PathBuf::from).collect())),
            Command::History => Ok(Request::History(arguments.map(PathBuf::from).collect())),
            Command::Sources => Ok(Request::Sources(arguments.collect())),
            Command::Beyond => arguments
                .next()
                .ok_or(Failure::Usage)
                .map(PathBuf::from)
                .map(|produced| Request::Beyond { produced, changed: arguments.map(PathBuf::from).collect() }),
        }
    }

    pub fn written(self, files: &impl Files, metadata: &Path) -> Result<Answer, Failure> {
        match self {
            Request::Affected(changed) => Ok(Answer::new(ExitCode::SUCCESS)
                .showing(Workspace::from(&Self::metadata(files, metadata)?).reached(&changed))),
            Request::Widened(changed) => Ok(Answer::new(ExitCode::SUCCESS)
                .showing(Workspace::from(&Self::metadata(files, metadata)?).reached(&changed).widened())),
            Request::History(changed) => Ok(Answer::new(ExitCode::SUCCESS)
                .showing(Workspace::from(&Self::metadata(files, metadata)?).history(&changed))),
            Request::Sources(roots) => Ok(Answer::new(ExitCode::SUCCESS)
                .showing(Workspace::from(&Self::metadata(files, metadata)?).sources(&roots))),
            Request::Beyond { produced, changed } => {
                Ok(Workspace::from(&Self::metadata(files, metadata)?).beyond(&produced, &changed).map_or_else(
                    || Answer::new(ExitCode::SUCCESS),
                    |path| Answer::new(ExitCode::SUCCESS).showing(path.display()).showing('\n'),
                ))
            }
        }
    }

    fn metadata(files: &impl Files, metadata: &Path) -> Result<Metadata, Failure> {
        let text = files
            .read_text(metadata)
            .map_err(|failure| Failure::Read { path: metadata.to_path_buf(), source: io::Error::from(failure) })?;
        Ok(serde_json::from_str::<Metadata>(text.as_ref())?.checked()?)
    }
}
