use std::str::FromStr;

use super::change::Change;
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

    pub(crate) fn flagging(self, change: &Change) -> Option<&Change> {
        match self {
            Check::Goldens => change.touching(Place::GOLDENS),
            Check::Scenarios => change.bringing_in(Place::SCENARIOS).map(|_| change),
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
