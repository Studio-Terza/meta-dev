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
use grammar_flag::Flags;
use store_ports::Sqlite;

use super::failure::Failure;
use command::{Command, Verb};
pub use command::{Crew, Unresolved};
pub use ground::Ground;
use persistence::Persistence;
use projects::Projects;
pub use projects::{Kept, Located};
pub use token::{Shaped, Token, Valued};

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
}

#[derive(Debug, Default)]
struct Words {
    verb: Option<Verb>,
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
    Value(Valued),
}

#[derive(Debug)]
enum Argument {
    Flag(Flag),
    Plain(OsString),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flag {
    Workspace,
    DryRun,
    All,
    Settings,
    Valued(Valued),
}

impl FromStr for Flag {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Flag, Failure> {
        Self::NAMED.iter().find(|(known, _)| *known == name).map(|(_, flag)| *flag).ok_or(Failure::Usage)
    }
}

impl Flag {
    const NAMED: [(&'static str, Flag); 8] = [
        ("--workspace", Flag::Workspace),
        ("--dry-run", Flag::DryRun),
        ("--all", Flag::All),
        ("--settings", Flag::Settings),
        ("--consumes", Flag::Valued(Valued::Consumes)),
        ("--artifact", Flag::Valued(Valued::Artifact)),
        ("--condition", Flag::Valued(Valued::Condition)),
        ("--reason", Flag::Valued(Valued::Reason)),
    ];

    fn owned(self, verb: Option<Verb>) -> Result<Flag, Failure> {
        [Flag::Workspace]
            .iter()
            .chain(verb.map_or(&[][..], Verb::options))
            .find(|owned| **owned == self)
            .copied()
            .ok_or(Failure::Usage)
    }

    fn expectation(self) -> Expecting {
        match self {
            Flag::Workspace => Expecting::Path,
            Flag::Valued(valued) => Expecting::Value(valued),
            Flag::DryRun | Flag::All | Flag::Settings => Expecting::Word,
        }
    }

    fn scope(self, current: Scope) -> Scope {
        match self {
            Flag::All => Scope::Everyone,
            Flag::Workspace | Flag::DryRun | Flag::Settings | Flag::Valued(_) => current,
        }
    }

    fn token(self) -> Option<Token> {
        match self {
            Flag::Workspace | Flag::All | Flag::Valued(_) => None,
            Flag::DryRun => Some(Token::Persisted(Persistence::DryRun)),
            Flag::Settings => Some(Token::Settings),
        }
    }
}

impl TryFrom<OsString> for Argument {
    type Error = Failure;

    fn try_from(argument: OsString) -> Result<Argument, Failure> {
        match argument.to_str().and_then(Flags::spelled) {
            Some(flag) => flag.parse().map(Argument::Flag),
            None => Ok(Argument::Plain(argument)),
        }
    }
}

impl Words {
    fn taking(self, argument: OsString) -> Result<Words, Failure> {
        match self.expecting {
            Expecting::Path => {
                Ok(Words { workspace: Some(PathBuf::from(argument)), expecting: Expecting::Word, ..self })
            }
            Expecting::Value(valued) => Ok(Words {
                spoken: self.spoken.into_iter().chain([Token::Valued(valued, argument)]).collect(),
                expecting: Expecting::Word,
                ..self
            }),
            Expecting::Word => Argument::try_from(argument).and_then(|read| self.worded(read)),
        }
    }

    fn worded(self, argument: Argument) -> Result<Words, Failure> {
        match argument {
            Argument::Flag(flag) => flag.owned(self.verb).map(|owned| self.flagged(owned)),
            Argument::Plain(word) => self.said(Token::Word(word)),
        }
    }

    fn said(self, word: Token) -> Result<Words, Failure> {
        match self.verb {
            None => word.text().ok_or(Failure::Usage)?.parse().map(|verb| Words { verb: Some(verb), ..self }),
            Some(_) => Ok(Words { spoken: self.spoken.into_iter().chain([word]).collect(), ..self }),
        }
    }

    fn flagged(self, flag: Flag) -> Words {
        Words {
            expecting: flag.expectation(),
            scope: flag.scope(self.scope),
            spoken: self.spoken.into_iter().chain(flag.token()).collect(),
            ..self
        }
    }
}

impl TryFrom<Words> for Request {
    type Error = Failure;

    fn try_from(words: Words) -> Result<Request, Failure> {
        match words.expecting {
            Expecting::Path | Expecting::Value(_) => Err(Failure::Usage),
            Expecting::Word => words
                .verb
                .ok_or(Failure::Usage)
                .and_then(|verb| verb.command(&words.spoken, words.scope))
                .map(|command| Request { command, workspace: words.workspace }),
        }
    }
}

impl Request {
    pub fn parsed(mut arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        arguments.try_fold(Words::default(), Words::taking).and_then(Request::try_from)
    }

    pub fn located(&self, environment: &impl Environment, files: &impl Files) -> Result<Located, Failure> {
        Projects::located(environment, files, self.workspace.as_deref().map(Path::new))
    }

    pub fn opened(&self, located: &Located, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        self.command.opened(located.kept(Kept::Store), opened)
    }

    pub fn command(self) -> Command {
        self.command
    }
}
