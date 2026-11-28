mod command;
mod ground;
mod ignored;
mod local;
mod persistence;
mod projects;
mod queue;

use std::ffi::OsString;
use std::path::PathBuf;

use console_ports::Variables;
use store_ports::Opens;

use super::failure::Failure;
use command::Command;
pub use command::{Answer, Asked, Crew, Tape};
pub use ground::Ground;
pub use local::{Local, Repository, Rooted};
use projects::Projects;
pub use projects::{Kept, Located};
pub use queue::{Cause, Held, InstallStep, Item, Landed, Named, Queue, Target, UnreadStep};

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
    harness: Option<PathBuf>,
}

#[derive(Debug)]
struct Words {
    spoken: Vec<OsString>,
    flagged: Option<PathBuf>,
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        let workspace = Words::heard(&arguments.collect::<Vec<OsString>>(), Words::WORKSPACE)?;
        let harness = Words::heard(&workspace.spoken, Words::HARNESS)?;
        let spoken = harness.spoken.iter().map(|word| word.to_str()).collect::<Option<Vec<&str>>>();
        let command = spoken.ok_or(Failure::Usage).and_then(|said| Command::spoken(&said))?;
        Ok(Request { command, workspace: workspace.flagged, harness: harness.flagged })
    }

    pub fn located(&self, variables: &Variables) -> Result<Located, Failure> {
        Projects::located(variables, self.workspace.as_deref(), self.harness.as_deref())
    }

    pub fn opened<O: Opens>(&self, located: &Located, opens: &O) -> O::Store {
        self.command.opened(located.kept(Kept::Store), opens)
    }

    pub fn command(&self) -> Command {
        self.command
    }
}

impl Words {
    const WORKSPACE: &'static str = "--workspace";
    const HARNESS: &'static str = "--harness";

    fn heard(given: &[OsString], flag: &str) -> Result<Words, Failure> {
        let found = given.iter().position(|word| word == flag);
        found.map_or(Ok(Words { spoken: given.to_vec(), flagged: None }), |at| {
            let (before, after) = given.split_at(at);
            match after {
                [_flag, path, rest @ ..] => Ok(Words {
                    spoken: before.iter().chain(rest).cloned().collect(),
                    flagged: Some(PathBuf::from(path)),
                }),
                [] | [_] => Err(Failure::Usage),
            }
        })
    }
}
