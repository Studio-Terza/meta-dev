mod boundary;

use std::fmt;
use std::iter;

use super::crates::{Crates, Origin};
use super::manifest::Kind;
use super::package::Package;
use super::unit_key::UnitKey;
pub(super) use boundary::Target;
use text_lines::OneLine;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Role {
    Library,
    HostBuild,
    BuildScript,
    BuildRun,
    LibraryTest,
    Target(OneLine),
    TestTarget(OneLine),
}

impl Role {
    const EVERY_ORIGIN: [Origin; 3] = [Origin::Workspace, Origin::ProcMacro, Origin::Registry];
    const LINKED_ORIGINS: [Origin; 2] = [Origin::Workspace, Origin::ProcMacro];
    const WORKSPACE_ORIGIN: [Origin; 1] = [Origin::Workspace];
    const NORMAL: [Kind; 1] = [Kind::Normal];
    const NORMAL_AND_DEV: [Kind; 2] = [Kind::Normal, Kind::Dev];
    const BUILD: [Kind; 1] = [Kind::Build];

    pub(super) fn uncached(&self) -> &'static [Origin] {
        match self {
            Role::BuildScript | Role::BuildRun => &Self::EVERY_ORIGIN,
            Role::HostBuild => &Self::LINKED_ORIGINS,
            Role::Library | Role::LibraryTest | Role::Target(_) | Role::TestTarget(_) => &Self::WORKSPACE_ORIGIN,
        }
    }

    pub(super) fn before(&self, key: &UnitKey, crates: &Crates) -> Vec<UnitKey> {
        match self {
            Role::BuildRun => vec![key.sibling(Role::BuildScript)],
            Role::BuildScript => Self::hosted(crates.dependencies(key.package(), &Self::BUILD)),
            Role::HostBuild => Self::hosted(crates.dependencies(key.package(), &Self::NORMAL))
                .into_iter()
                .chain(iter::once(key.sibling(Role::BuildRun)))
                .collect(),
            Role::Library => Self::checked(key, crates, &Self::NORMAL),
            Role::LibraryTest => Self::checked(key, crates, &Self::NORMAL_AND_DEV),
            Role::Target(_) => Self::beside_library(key, Self::checked(key, crates, &Self::NORMAL)),
            Role::TestTarget(_) => Self::beside_library(key, Self::checked(key, crates, &Self::NORMAL_AND_DEV)),
        }
    }

    fn hosted(packages: Vec<&Package>) -> Vec<UnitKey> {
        packages.into_iter().map(|package| UnitKey::new(package.clone(), Role::HostBuild)).collect()
    }

    fn checked(key: &UnitKey, crates: &Crates, kinds: &[Kind]) -> Vec<UnitKey> {
        crates
            .dependencies(key.package(), kinds)
            .into_iter()
            .map(|package| crates.linked(package))
            .chain(iter::once(key.sibling(Role::BuildRun)))
            .collect()
    }

    fn beside_library(key: &UnitKey, checked: Vec<UnitKey>) -> Vec<UnitKey> {
        checked.into_iter().chain(iter::once(key.sibling(Role::Library))).collect()
    }
}

impl fmt::Display for Role {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Library => formatter.write_str(" (check)"),
            Role::HostBuild => formatter.write_str(" (build)"),
            Role::BuildScript => formatter.write_str(" build-script"),
            Role::BuildRun => formatter.write_str(" build-script (run)"),
            Role::LibraryTest => formatter.write_str(" (check-test)"),
            Role::Target(target) | Role::TestTarget(target) => write!(formatter, "{target}"),
        }
    }
}
