use std::fmt;

use super::change::Change;
use super::place::Place;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    Goldens,
    Scenarios,
}

impl Check {
    pub(crate) const ALL: [Check; 2] = [Check::Goldens, Check::Scenarios];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Check::Goldens => "golden-change",
            Check::Scenarios => "new-scenario",
        }
    }

    pub(crate) fn flagged(self, changes: &[Change]) -> Vec<&Change> {
        match self {
            Check::Goldens => changes.iter().filter_map(|change| change.touching(Place::GOLDENS)).collect(),
            Check::Scenarios => {
                changes.iter().filter(|change| change.bringing_in(Place::SCENARIOS).is_some()).collect()
            }
        }
    }
}

impl fmt::Display for Check {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::Goldens => write!(
                formatter,
                "this PR changes goldens; a person reviews the change and adds the label {}",
                self.label()
            ),
            Check::Scenarios => write!(
                formatter,
                "this PR adds files under tests/fixtures or tests/e2e; a person reviews the scenario and adds the label {}",
                self.label()
            ),
        }
    }
}
