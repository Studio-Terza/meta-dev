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
        changes.iter().filter_map(|change| self.flagging(change)).collect()
    }

    pub(crate) fn flagging(self, change: &Change) -> Option<&Change> {
        match self {
            Check::Goldens => change.touching(Place::GOLDENS),
            Check::Scenarios => change.bringing_in(Place::SCENARIOS).map(|_| change),
        }
    }
}
