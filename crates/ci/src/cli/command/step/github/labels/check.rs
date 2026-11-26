use std::path::Path;

use strum::{Display, EnumString, VariantArray};

use super::change::Change;
use super::fault::Fault;
use super::hub::Purpose;
use super::place::Place;

#[derive(Display, EnumString, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    #[strum(serialize = "golden-change")]
    Goldens,
    #[strum(serialize = "new-scenario")]
    Scenarios,
}

impl Check {
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
