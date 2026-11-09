use grammar::unit_label::{Fixed, Mode, Target, Unit};

use super::Role;

#[derive(Debug, thiserror::Error)]
#[error("a unit label cargo's timings never give a clippy unit")]
pub(in super::super) struct Unlabeled;

impl TryFrom<String> for Role {
    type Error = Unlabeled;

    fn try_from(text: String) -> Result<Role, Unlabeled> {
        Role::labelled(&text).ok_or(Unlabeled)
    }
}

impl Unit for Role {
    fn fixed(fixed: Fixed) -> Role {
        match fixed {
            Fixed::Built => Role::HostBuild,
            Fixed::Checked => Role::Library,
            Fixed::CheckedTest => Role::LibraryTest,
            Fixed::BuildScript => Role::BuildScript,
            Fixed::BuildRun => Role::BuildRun,
        }
    }

    fn target(target: Target) -> Role {
        match target.mode() {
            Mode::Check => Role::Target(target),
            Mode::CheckTest => Role::TestTarget(target),
        }
    }
}
