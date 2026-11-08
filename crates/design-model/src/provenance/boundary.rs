use std::fmt;
use std::str::FromStr;

use grammar::provenance::{Originating, Origins};
use newtype::{Newtype, Spelled};

use crate::{Entered, NotesLine, RunId};

#[derive(Spelled, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Lead(Entered),
    Agent(RunId),
    Import(NotesLine),
}

#[derive(Debug, thiserror::Error)]
#[error(
    "{0:?} is no provenance: one of init, apply, role, inheritance, agent- and a run number, as in agent-7, or import- and a notes line, as in import-12"
)]
pub struct Unprovenanced(String);

#[derive(Clone, Copy, Debug)]
struct Originated;

impl From<Entered> for Provenance {
    fn from(entered: Entered) -> Provenance {
        Provenance::Lead(entered)
    }
}

impl Originating for Originated {
    type Output = Provenance;

    fn agent(self, run: i64) -> Provenance {
        Provenance::Agent(RunId::new(run))
    }

    fn import(self, line: usize) -> Provenance {
        Provenance::Import(NotesLine::new(line))
    }
}

impl Provenance {
    #[must_use]
    pub fn imported(self) -> Option<NotesLine> {
        match self {
            Provenance::Import(line) => Some(line),
            Provenance::Lead(_) | Provenance::Agent(_) => None,
        }
    }
}

impl fmt::Display for Provenance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provenance::Lead(entered) => entered.fmt(formatter),
            Provenance::Agent(run) => formatter.write_str(&Origins::agent(run.into_inner())),
            Provenance::Import(line) => formatter.write_str(&Origins::import(line.into_inner())),
        }
    }
}

impl FromStr for Provenance {
    type Err = Unprovenanced;

    fn from_str(text: &str) -> Result<Provenance, Unprovenanced> {
        let origin = || Origins::read(text, Originated);
        text.parse::<Entered>().map(Provenance::Lead).ok().or_else(origin).ok_or_else(|| Unprovenanced(text.to_owned()))
    }
}
