mod ingest;
mod printed;
mod shown;

use std::ffi::OsString;
use std::io::Write;
use std::str::FromStr;

use transcript_ports::{Transcripts, Uuid};

use crate::Outcome;
use crate::cli::failure::Failure;
use ingest::Ingest;
use printed::Printed;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Ingest,
    Show(Uuid),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verb {
    Ingest,
    Show,
}

impl FromStr for Verb {
    type Err = Failure;

    fn from_str(name: &str) -> Result<Verb, Failure> {
        Self::NAMED.iter().find_map(|(known, verb)| (*known == name).then_some(*verb)).ok_or(Failure::Usage)
    }
}

impl Verb {
    const NAMED: [(&'static str, Verb); 2] = [("ingest", Verb::Ingest), ("show", Verb::Show)];

    fn command(self, rest: &[OsString]) -> Result<Command, Failure> {
        match self {
            Verb::Ingest => Ok(Command::Ingest),
            Verb::Show => rest
                .first()
                .and_then(|word| word.to_str())
                .ok_or(Failure::Usage)
                .map(|uuid| Command::Show(Uuid::new(uuid.to_owned()))),
        }
    }
}

impl TryFrom<&[OsString]> for Command {
    type Error = Failure;

    fn try_from(words: &[OsString]) -> Result<Command, Failure> {
        words
            .first()
            .and_then(|word| word.to_str())
            .ok_or(Failure::Usage)
            .and_then(str::parse::<Verb>)
            .and_then(|verb| verb.command(words.get(1..).unwrap_or_default()))
    }
}

impl Command {
    pub fn written(&self, port: &impl Transcripts, out: &mut impl Write) -> Result<Outcome, Failure> {
        match self {
            Command::Ingest => {
                Ingest::read(port).and_then(|tally| Printed::shown(out, &tally)).map(|_printed| Outcome::Ingested)
            }
            Command::Show(uuid) => {
                Shown::find(port, uuid).and_then(|shown| Printed::shown(out, &shown)).map(|_printed| Outcome::Shown)
            }
        }
    }
}
