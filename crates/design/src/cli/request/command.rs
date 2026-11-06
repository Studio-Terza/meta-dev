mod apply;
mod ingest;
mod init;
mod printed;
mod search;
mod structure;
mod subject;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use design_store::Scope;
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use grammar::search::Terms;
use store_ports::{Sqlite, Store};

use super::persistence::Persistence;
use super::{Crew, Ground, Token};
use crate::Outcome;
use crate::cli::failure::Failure;
pub use apply::{Application, Applied, Unresolved};
use ingest::Ingest;
use init::Init;
use printed::Printed;
use search::Search;
use structure::Structure;
use subject::Subject;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest(Persistence),
    Show(Subject),
    Search(Terms, Scope),
    Init(Init),
    Apply(PathBuf),
    Structure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verb {
    Ingest,
    Show,
    Search,
    Init,
    Apply,
    Structure,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 6] = [
        ("ingest", Verb::Ingest),
        ("show", Verb::Show),
        ("search", Verb::Search),
        ("init", Verb::Init),
        ("apply", Verb::Apply),
        ("structure", Verb::Structure),
    ];

    fn command(self, rest: &[Token], scope: Scope) -> Result<Command, Failure> {
        match self {
            Verb::Ingest => Persistence::try_from(rest).map(Command::Ingest),
            Verb::Show => Token::only(rest).and_then(Subject::try_from).map(Command::Show),
            Verb::Search => rest
                .iter()
                .map(Token::text)
                .collect::<Option<Vec<&str>>>()
                .and_then(Terms::lexed)
                .ok_or(Failure::Usage)
                .map(|terms| Command::Search(terms, scope)),
            Verb::Init => Init::try_from(rest).map(Command::Init),
            Verb::Apply => Token::only(rest).map(PathBuf::from).map(Command::Apply),
            Verb::Structure => rest.first().map_or(Ok(Command::Structure), |_extra| Err(Failure::Usage)),
        }
    }
}

impl Command {
    pub fn spoken(words: &[Token], scope: Scope) -> Result<Command, Failure> {
        words
            .first()
            .and_then(Token::text)
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Verb>)
            .and_then(|verb| verb.command(words.get(1..).unwrap_or_default(), scope))
    }

    pub fn opened(&self, path: PathBuf, opened: impl FnOnce(PathBuf) -> Sqlite) -> Sqlite {
        match self {
            Command::Ingest(persistence) => persistence.opened(path, opened),
            Command::Show(_) | Command::Search(..) | Command::Init(_) | Command::Apply(_) | Command::Structure => {
                opened(path)
            }
        }
    }

    pub fn written<F: Files, S: Store, D: Dispatch, C: Clock>(
        self,
        ground: Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Command::Ingest(_) => Ingest::read(ground.store(), &ground.transcripts())
                .and_then(|tally| Printed::shown(out, &tally))
                .map(|_printed| Outcome::Ingested),
            Command::Show(subject) => subject.shown(ground.store(), out).map(|_printed| Outcome::Shown),
            Command::Search(terms, scope) => Search::found(ground.store(), terms, scope)
                .and_then(|search| Printed::shown(out, &search))
                .map(|_printed| Outcome::Searched),
            Command::Init(init) => init
                .planted(&ground)
                .and_then(|initialised| Printed::shown(out, &initialised))
                .map(|_printed| Outcome::Initialised),
            Command::Apply(path) => Application::read(&ground, &path).and_then(|application| application.reported(out)),
            Command::Structure => {
                Structure::new(&ground, crew).worked().and_then(|structured| structured.reported(out))
            }
        }
    }
}
