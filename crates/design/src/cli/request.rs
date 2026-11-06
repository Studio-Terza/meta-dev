mod command;
mod ground;
mod persistence;
mod projects;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use console_ports::Environment;
use design_model::Project;
use design_store::Scope;

use super::failure::Failure;
use command::Command;
pub use ground::Ground;
use persistence::Persistence;
use projects::Located;
use projects::Projects;

#[derive(Debug)]
pub struct Request {
    command: Command,
    workspace: Option<PathBuf>,
    persistence: Persistence,
}

#[derive(Debug, Default)]
struct Words {
    plain: Vec<OsString>,
    workspace: Option<PathBuf>,
    consumes: Option<OsString>,
    persistence: Persistence,
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

    fn persistence(&self, current: Persistence) -> Persistence {
        match self {
            Argument::DryRun => Persistence::DryRun,
            Argument::Workspace | Argument::All | Argument::Consumes | Argument::Plain(_) => current,
        }
    }

    fn scope(&self, current: Scope) -> Scope {
        match self {
            Argument::All => Scope::Everyone,
            Argument::Workspace | Argument::DryRun | Argument::Consumes | Argument::Plain(_) => current,
        }
    }

    fn plain(self) -> Option<OsString> {
        match self {
            Argument::Workspace | Argument::DryRun | Argument::All | Argument::Consumes => None,
            Argument::Plain(word) => Some(word),
        }
    }
}

impl Words {
    fn taking(self, argument: OsString) -> Words {
        match self.expecting {
            Expecting::Path => self.placed(PathBuf::from(argument)),
            Expecting::Project => Words { consumes: Some(argument), expecting: Expecting::Word, ..self },
            Expecting::Word => self.worded(Argument::from(argument)),
        }
    }

    fn placed(self, path: PathBuf) -> Words {
        Words { workspace: Some(path), expecting: Expecting::Word, ..self }
    }

    fn worded(self, argument: Argument) -> Words {
        Words {
            expecting: argument.expectation(),
            persistence: argument.persistence(self.persistence),
            scope: argument.scope(self.scope),
            plain: self.plain.into_iter().chain(argument.plain()).collect(),
            ..self
        }
    }

    fn consumed(consumes: Option<OsString>) -> Result<Option<Project>, Failure> {
        consumes
            .map(|word| word.into_string().map_err(|_unreadable| Failure::Usage).and_then(|text| Ok(text.parse()?)))
            .transpose()
    }
}

impl TryFrom<Words> for Request {
    type Error = Failure;

    fn try_from(words: Words) -> Result<Request, Failure> {
        match words.expecting {
            Expecting::Path | Expecting::Project => Err(Failure::Usage),
            Expecting::Word => Command::spoken(&words.plain, words.scope, Words::consumed(words.consumes)?)
                .map(|command| Request { command, workspace: words.workspace, persistence: words.persistence }),
        }
    }
}

impl Request {
    pub fn parsed(arguments: impl Iterator<Item = OsString>) -> Result<Request, Failure> {
        Request::try_from(arguments.fold(Words::default(), Words::taking))
    }

    pub fn located(&self, environment: &impl Environment) -> Result<Located, Failure> {
        Projects::located(environment, self.workspace.as_deref().map(Path::new), self.persistence)
    }

    pub fn command(self) -> Command {
        self.command
    }
}
