use std::fmt;
use std::str::FromStr;

use grammar::provenance::Agents;
use newtype::{Spelled, Wrapping};

use crate::{Entered, RunId};

#[derive(Spelled, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    Lead(Entered),
    Agent(RunId),
}

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is no provenance: one of init, apply, role, inheritance, or agent- and a run number, as in agent-7")]
pub struct Unprovenanced(String);

impl From<Entered> for Provenance {
    fn from(entered: Entered) -> Provenance {
        Provenance::Lead(entered)
    }
}

impl fmt::Display for Provenance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provenance::Lead(entered) => entered.fmt(formatter),
            Provenance::Agent(run) => formatter.write_str(&Agents::spelled(run.into_inner())),
        }
    }
}

impl FromStr for Provenance {
    type Err = Unprovenanced;

    fn from_str(text: &str) -> Result<Provenance, Unprovenanced> {
        let agent = || Agents::numbered(text).map(|run| Provenance::Agent(RunId::new(run)));
        text.parse::<Entered>().map(Provenance::Lead).ok().or_else(agent).ok_or_else(|| Unprovenanced(text.to_owned()))
    }
}
