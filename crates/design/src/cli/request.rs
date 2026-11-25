mod command;
mod ground;
mod persistence;
mod projects;
mod said;
mod token;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use algebra::Monoid;
use console_ports::Environment;
use design_store::Scope;
use file_ports::Files;
use store_ports::Opens;

use super::failure::Failure;
use command::{Command, Verb};
pub use command::{Crew, Unresolved};
pub use ground::Ground;
use persistence::Persistence;
use projects::Projects;
pub use projects::{Kept, Located};
use said::{Heard, Said, Saying, Wanting};
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

    fn heard(self, verb: Option<Verb>) -> Result<Heard, Failure> {
        [Flag::Workspace]
            .iter()
            .chain(verb.map_or(&[][..], Verb::options))
            .find(|owned| **owned == self)
            .ok_or(Failure::Usage)
            .map(|owned| owned.listened())
    }

    fn listened(self) -> Heard {
        match self {
            Flag::Workspace => Heard::Wanting(Wanting::Path),
            Flag::Valued(valued) => Heard::Wanting(Wanting::Value(valued)),
            Flag::DryRun | Flag::All | Flag::Settings => Heard::Whole(Said::Flagged(self)),
        }
    }

    fn scope(self) -> Scope {
        match self {
            Flag::All => Scope::Everyone,
            Flag::Workspace | Flag::DryRun | Flag::Settings | Flag::Valued(_) => Scope::default(),
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

impl TryFrom<Words> for Request {
    type Error = Failure;

    fn try_from(words: Words) -> Result<Request, Failure> {
        words
            .verb
            .ok_or(Failure::Usage)
            .and_then(|verb| verb.command(&words.spoken, words.scope))
            .map(|command| Request { command, workspace: words.workspace })
    }
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        Saying::said(arguments)
            .collect::<Result<Vec<Said>, Failure>>()
            .map(|said| Words::concat(said.into_iter().map(Words::from)))
            .and_then(Request::try_from)
    }

    pub fn located(&self, environment: &impl Environment, files: &impl Files) -> Result<Located, Failure> {
        Projects::located(environment, files, self.workspace.as_deref().map(Path::new))
    }

    pub fn opened<O: Opens>(&self, located: &Located, opens: &O) -> O::Store {
        self.command.opened(located.kept(Kept::Store), opens)
    }

    pub fn command(self) -> Command {
        self.command
    }
}
