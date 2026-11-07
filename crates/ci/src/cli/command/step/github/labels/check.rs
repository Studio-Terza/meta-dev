use std::str::FromStr;

use super::change::Change;
use super::fault::Fault;
use super::hub::PullRequest;
use super::place::Place;

#[derive(Debug, PartialEq, Eq)]
pub struct NotACheck;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Goldens,
    Scenarios,
}

impl Check {
    pub(crate) const ALL: [Self; 2] = [Self::Goldens, Self::Scenarios];
    pub(crate) const GOLDENS: &'static str = "golden-change";
    pub(crate) const SCENARIOS: &'static str = "new-scenario";

    pub(crate) fn waived_by(self, refactor: Option<&PullRequest>) -> Option<Fault> {
        match self {
            Self::Goldens => refactor.map(|_| Fault::Refactor),
            Self::Scenarios => None,
        }
    }

    pub(crate) fn flagging(self, change: &Change) -> Option<&Change> {
        match self {
            Self::Goldens => change.touching(Place::GOLDENS),
            Self::Scenarios => change.bringing_in(Place::SCENARIOS).map(|_| change),
        }
    }
}

impl FromStr for Check {
    type Err = NotACheck;

    fn from_str(label: &str) -> Result<Self, NotACheck> {
        match label {
            Self::GOLDENS => Ok(Self::Goldens),
            Self::SCENARIOS => Ok(Self::Scenarios),
            _ => Err(NotACheck),
        }
    }
}
