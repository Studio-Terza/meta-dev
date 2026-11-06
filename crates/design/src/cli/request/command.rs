mod ingest;
mod printed;
mod search;
mod shown;

use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;

use design_store::Scope;
use grammar::search::Terms;
use store_ports::Store;
use transcript_ports::{Transcripts, Uuid};

use crate::Outcome;
use crate::cli::failure::Failure;
use ingest::Ingest;
use printed::Printed;
use search::Search;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest,
    Show(Uuid),
    Search(Terms, Scope),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verb {
    Ingest,
    Show,
    Search,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 3] = [("ingest", Verb::Ingest), ("show", Verb::Show), ("search", Verb::Search)];

    fn command(self, rest: &[OsString], scope: Scope) -> Result<Command, Failure> {
        match self {
            Verb::Ingest => Ok(Command::Ingest),
            Verb::Show => rest
                .first()
                .and_then(|word| word.to_str())
                .ok_or(Failure::Usage)
                .map(|uuid| Command::Show(Uuid::new(uuid.to_owned()))),
            Verb::Search => rest
                .iter()
                .map(|word| word.to_str())
                .collect::<Option<Vec<&str>>>()
                .and_then(Terms::lexed)
                .ok_or(Failure::Usage)
                .map(|terms| Command::Search(terms, scope)),
        }
    }
}

impl Command {
    pub fn spoken(words: &[OsString], scope: Scope) -> Result<Command, Failure> {
        words
            .first()
            .and_then(|word| word.to_str())
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Verb>)
            .and_then(|verb| verb.command(words.get(1..).unwrap_or_default(), scope))
    }

    pub fn written(
        self,
        transcripts: &impl Transcripts,
        store: &impl Store,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self {
            Command::Ingest => Ingest::read(transcripts, store)
                .and_then(|tally| Printed::shown(out, &tally))
                .map(|_printed| Outcome::Ingested),
            Command::Show(uuid) => {
                Shown::find(store, &uuid).and_then(|shown| Printed::shown(out, &shown)).map(|_printed| Outcome::Shown)
            }
            Command::Search(terms, scope) => Search::found(store, terms, scope)
                .and_then(|search| Printed::shown(out, &search))
                .map(|_printed| Outcome::Searched),
        }
    }
}
