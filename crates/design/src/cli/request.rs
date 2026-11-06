mod command;
mod projects;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use console_ports::Environment;

use super::failure::Failure;
use command::Command;
use projects::Projects;

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
}

#[derive(Debug, Default)]
struct Words {
    plain: Vec<OsString>,
    workspace: Option<PathBuf>,
    expecting: Expecting,
}

#[derive(Debug, Default, Clone, Copy)]
enum Expecting {
    #[default]
    Word,
    Path,
}

#[derive(Debug)]
enum Argument {
    Workspace,
    Plain(OsString),
}

#[derive(Clone, Copy, Debug)]
enum Flag {
    Workspace,
}

impl FromStr for Flag {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Flag, Failure> {
        Self::NAMED.iter().find(|(known, _)| *known == name).map(|(_, flag)| *flag).ok_or(Failure::Usage)
    }
}

impl Flag {
    const NAMED: [(&'static str, Flag); 1] = [("--workspace", Flag::Workspace)];
}

impl From<OsString> for Argument {
    fn from(argument: OsString) -> Argument {
        match argument.to_str().and_then(|word| word.parse::<Flag>().ok()) {
            Some(Flag::Workspace) => Argument::Workspace,
            None => Argument::Plain(argument),
        }
    }
}

impl Argument {
    fn expectation(&self) -> Expecting {
        match self {
            Argument::Workspace => Expecting::Path,
            Argument::Plain(_) => Expecting::Word,
        }
    }

    fn plain(self) -> Option<OsString> {
        match self {
            Argument::Workspace => None,
            Argument::Plain(word) => Some(word),
        }
    }
}

impl Words {
    fn taking(self, argument: OsString) -> Words {
        match self.expecting {
            Expecting::Path => self.placed(PathBuf::from(argument)),
            Expecting::Word => self.worded(Argument::from(argument)),
        }
    }

    fn placed(self, path: PathBuf) -> Words {
        Words { workspace: Some(path), expecting: Expecting::Word, ..self }
    }

    fn worded(self, argument: Argument) -> Words {
        Words {
            expecting: argument.expectation(),
            plain: self.plain.into_iter().chain(argument.plain()).collect(),
            ..self
        }
    }
}

impl TryFrom<Words> for Request {
    type Error = Failure;

    fn try_from(words: Words) -> Result<Request, Failure> {
        match words.expecting {
            Expecting::Path => Err(Failure::Usage),
            Expecting::Word => {
                Command::try_from(words.plain.as_slice()).map(|command| Request { command, workspace: words.workspace })
            }
        }
    }
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        Request::try_from(arguments.fold(Words::default(), Words::taking))
    }

    pub fn directory(&self, environment: &impl Environment) -> Result<PathBuf, Failure> {
        Projects::directory(environment, self.workspace.as_deref().map(Path::new))
    }

    pub fn command(&self) -> &Command {
        &self.command
    }
}
