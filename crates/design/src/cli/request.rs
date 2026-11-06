mod command;
mod ground;
mod persistence;
mod projects;
mod token;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use console_ports::Environment;
use design_store::Scope;
use file_ports::Files;
use store_ports::Sqlite;

use super::failure::Failure;
use command::Command;
pub use command::{Applied, Unresolved};
pub use ground::Ground;
use persistence::Persistence;
use projects::Located;
use projects::Projects;
pub use token::Token;

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
}

#[derive(Debug, Default)]
struct Words {
    spoken: Vec<Token>,
    workspace: Option<PathBuf>,
    scope: Scope,
    expecting: Expecting,
}

#[derive(Debug, Default, Clone, Copy)]
enum Expecting {
    #[default]
    Word,
    Path,
    Project,
}

#[derive(Debug)]
enum Argument {
    Workspace,
    DryRun,
    All,
    Consumes,
    Plain(OsString),
}

#[derive(Clone, Copy, Debug)]
enum Flag {
    Workspace,
    DryRun,
    All,
    Consumes,
}

impl FromStr for Flag {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Flag, Failure> {
        Self::NAMED.iter().find(|(known, _)| *known == name).map(|(_, flag)| *flag).ok_or(Failure::Usage)
    }
}

impl Flag {
    const NAMED: [(&'static str, Flag); 4] = [
        ("--workspace", Flag::Workspace),
        ("--dry-run", Flag::DryRun),
        ("--all", Flag::All),
        ("--consumes", Flag::Consumes),
    ];
}

impl From<OsString> for Argument {
    fn from(argument: OsString) -> Argument {
        match argument.to_str().and_then(|word| word.parse::<Flag>().ok()) {
            Some(Flag::Workspace) => Argument::Workspace,
            Some(Flag::DryRun) => Argument::DryRun,
            Some(Flag::All) => Argument::All,
            Some(Flag::Consumes) => Argument::Consumes,
            None => Argument::Plain(argument),
        }
    }
}

impl Argument {
    fn expectation(&self) -> Expecting {
        match self {
            Argument::Workspace => Expecting::Path,
            Argument::Consumes => Expecting::Project,
            Argument::DryRun | Argument::All | Argument::Plain(_) => Expecting::Word,
        }
    }

    fn scope(&self, current: Scope) -> Scope {
        match self {
            Argument::All => Scope::Everyone,
            Argument::Workspace | Argument::DryRun | Argument::Consumes | Argument::Plain(_) => current,
        }
    }

    fn token(self) -> Option<Token> {
        match self {
            Argument::Workspace | Argument::All | Argument::Consumes => None,
            Argument::DryRun => Some(Token::Persisted(Persistence::DryRun)),
            Argument::Plain(word) => Some(Token::Word(word)),
        }
    }
}

impl Words {
    fn taking(self, argument: OsString) -> Words {
        match self.expecting {
            Expecting::Path => self.placed(PathBuf::from(argument)),
            Expecting::Project => Words {
                spoken: self.spoken.into_iter().chain([Token::Consumes(argument)]).collect(),
                expecting: Expecting::Word,
                ..self
            },
            Expecting::Word => self.worded(Argument::from(argument)),
        }
    }

    fn placed(self, path: PathBuf) -> Words {
        Words { workspace: Some(path), expecting: Expecting::Word, ..self }
    }

    fn worded(self, argument: Argument) -> Words {
        Words {
            expecting: argument.expectation(),
            scope: argument.scope(self.scope),
            spoken: self.spoken.into_iter().chain(argument.token()).collect(),
            ..self
        }
    }
}

impl TryFrom<Words> for Request {
    type Error = Failure;

    fn try_from(words: Words) -> Result<Request, Failure> {
        match words.expecting {
            Expecting::Path | Expecting::Project => Err(Failure::Usage),
            Expecting::Word => Command::spoken(&words.spoken, words.scope)
                .map(|command| Request { command, workspace: words.workspace }),
        }
    }
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        Request::try_from(arguments.fold(Words::default(), Words::taking))
    }

    pub fn located(&self, environment: &impl Environment, files: &impl Files) -> Result<Located, Failure> {
        Projects::located(environment, files, self.workspace.as_deref().map(Path::new))
    }

    pub fn opened(&self, located: &Located, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        self.command.opened(located.store(), opened)
    }

    pub fn command(self) -> Command {
        self.command
    }
}
