use grammar::unit_label::{Fixed, Mode, Target, Unit};

use super::UnitRole;

#[derive(Debug, thiserror::Error)]
#[error("a unit label cargo's timings never give a clippy unit")]
pub(in super::super) struct Unlabeled;

impl TryFrom<String> for UnitRole {
    type Error = Unlabeled;

    fn try_from(text: String) -> Result<UnitRole, Unlabeled> {
        UnitRole::labelled(&text).ok_or(Unlabeled)
    }
}

impl Unit for UnitRole {
    fn fixed(fixed: Fixed) -> UnitRole {
        match fixed {
            Fixed::Built => UnitRole::HostBuild,
            Fixed::Checked => UnitRole::Library,
            Fixed::CheckedTest => UnitRole::LibraryTest,
            Fixed::BuildScript => UnitRole::BuildScript,
            Fixed::BuildRun => UnitRole::BuildRun,
        }
    }

    fn target(target: Target) -> UnitRole {
        match target.mode() {
            Mode::Check => UnitRole::Target(target),
            Mode::CheckTest => UnitRole::TestTarget(target),
        }
    }
}
