use std::path::Path;
use std::str::FromStr;

use super::change::Change;
use super::fault::Fault;
use super::hub::Purpose;
use super::place::Place;

#[derive(Debug, PartialEq, Eq)]
pub struct NotACheck;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Goldens,
    Scenarios,
}

impl Check {
    pub(crate) const ALL: [Check; 2] = [Check::Goldens, Check::Scenarios];
    pub(crate) const GOLDENS: &'static str = "golden-change";
    pub(crate) const SCENARIOS: &'static str = "new-scenario";

    pub(crate) fn waived_by(self, purpose: Purpose) -> Option<Fault> {
        match self {
            Check::Goldens => purpose.waiver(),
            Check::Scenarios => None,
        }
    }

    pub(crate) fn flagging(self, change: &Change) -> Option<&Path> {
        match self {
            Check::Goldens => change.paths().into_iter().find(|path| Place::GOLDENS.containing(path).is_some()),
            Check::Scenarios => change.bringing_in(Place::SCENARIOS),
        }
    }
}

impl FromStr for Check {
    type Err = NotACheck;

    fn from_str(label: &str) -> Result<Check, NotACheck> {
        match label {
            Self::GOLDENS => Ok(Check::Goldens),
            Self::SCENARIOS => Ok(Check::Scenarios),
            _ => Err(NotACheck),
        }
    }
}
