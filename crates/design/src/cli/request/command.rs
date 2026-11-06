mod ingest;
mod init;
mod printed;
mod search;
mod subject;

use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;

use design_model::Project;
use design_store::Scope;
use file_ports::Files;
use grammar::search::Terms;
use store_ports::Store;

use super::Ground;
use crate::Outcome;
use crate::cli::failure::Failure;
use ingest::Ingest;
use init::Init;
use printed::Printed;
use search::Search;
use subject::Subject;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest,
    Show(Subject),
    Search(Terms, Scope),
    Init(Init),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verb {
    Ingest,
    Show,
    Search,
    Init,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 4] =
        [("ingest", Verb::Ingest), ("show", Verb::Show), ("search", Verb::Search), ("init", Verb::Init)];

    fn command(self, rest: &[OsString], scope: Scope, consumes: Option<Project>) -> Result<Command, Failure> {
        match self {
            Verb::Ingest => Ok(Command::Ingest),
            Verb::Show => rest
                .first()
                .and_then(|word| word.to_str())
                .ok_or(Failure::Usage)
                .map(|word| Command::Show(Subject::from(word))),
            Verb::Search => rest
                .iter()
                .map(|word| word.to_str())
                .collect::<Option<Vec<&str>>>()
                .and_then(Terms::lexed)
                .ok_or(Failure::Usage)
                .map(|terms| Command::Search(terms, scope)),
            Verb::Init => rest
                .first()
                .and_then(|word| word.to_str())
                .ok_or(Failure::Usage)
                .and_then(|word| Ok(word.parse::<Project>()?))
                .map(|project| Command::Init(Init::new(project, consumes))),
        }
    }
}

impl Command {
    pub fn spoken(words: &[OsString], scope: Scope, consumes: Option<Project>) -> Result<Command, Failure> {
        words
            .first()
            .and_then(|word| word.to_str())
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Verb>)
            .and_then(|verb| verb.command(words.get(1..).unwrap_or_default(), scope, consumes))
    }

    pub fn written<F: Files, S: Store>(
        self,
        ground: Ground<'_, F, S>,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Command::Ingest => Ingest::read(ground.store(), &ground.transcripts())
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
        }
    }
}
