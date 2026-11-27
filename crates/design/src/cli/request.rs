mod command;
mod ground;
mod ignored;
mod local;
mod persistence;
mod projects;

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use console_ports::Environment;
use ports::store::Opens;

use super::failure::Failure;
use command::Command;
pub use command::Crew;
pub use ground::Ground;
pub use local::{Local, Repository};
use projects::Projects;
pub use projects::{Kept, Located};

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
}

#[derive(Debug)]
struct Words {
    spoken: Vec<OsString>,
    workspace: Option<PathBuf>,
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        let words = Words::heard(&arguments.collect::<Vec<OsString>>())?;
        let spoken = words.spoken.iter().map(|word| word.to_str()).collect::<Option<Vec<&str>>>();
        let command = spoken.ok_or(Failure::Usage).and_then(|said| Command::spoken(&said))?;
        Ok(Request { command, workspace: words.workspace })
    }

    pub fn located(&self, environment: &impl Environment) -> Result<Located, Failure> {
        Projects::located(environment, self.workspace.as_deref().map(Path::new))
    }

    pub fn opened<O: Opens>(&self, located: &Located, opens: &O) -> O::Store {
        self.command.opened(located.kept(Kept::Store), opens)
    }

    pub fn command(self) -> Command {
        self.command
    }
}

impl Words {
    const WORKSPACE: &'static str = "--workspace";

    fn heard(given: &[OsString]) -> Result<Words, Failure> {
        let flagged = given.iter().position(|word| word == Self::WORKSPACE);
        flagged.map_or(Ok(Words { spoken: given.to_vec(), workspace: None }), |at| {
            let (before, after) = given.split_at(at);
            match after {
                [_flag, path, rest @ ..] => Ok(Words {
                    spoken: before.iter().chain(rest).cloned().collect(),
                    workspace: Some(PathBuf::from(path)),
                }),
                [] | [_] => Err(Failure::Usage),
            }
        })
    }
}
