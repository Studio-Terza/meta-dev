mod boundary;

use std::fmt;
use std::iter;

use super::crates::{Crates, Origin};
use super::manifest::Kind;
use super::package::Package;
use super::unit_key::UnitKey;
use grammar_unit_label::Target;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub(super) enum UnitRole {
    Library,
    HostBuild,
    BuildScript,
    BuildRun,
    LibraryTest,
    Target(Target),
    TestTarget(Target),
}

impl UnitRole {
    const EVERY_ORIGIN: [Origin; 3] = [Origin::Workspace, Origin::ProcMacro, Origin::Registry];
    const LINKED_ORIGINS: [Origin; 2] = [Origin::Workspace, Origin::ProcMacro];
    const WORKSPACE_ORIGIN: [Origin; 1] = [Origin::Workspace];
    const NORMAL: [Kind; 1] = [Kind::Normal];
    const NORMAL_AND_DEV: [Kind; 2] = [Kind::Normal, Kind::Dev];
    const BUILD: [Kind; 1] = [Kind::Build];

    pub(super) fn uncached(&self) -> &'static [Origin] {
        match self {
            UnitRole::BuildScript | UnitRole::BuildRun => &Self::EVERY_ORIGIN,
            UnitRole::HostBuild => &Self::LINKED_ORIGINS,
            UnitRole::Library | UnitRole::LibraryTest | UnitRole::Target(_) | UnitRole::TestTarget(_) => {
                &Self::WORKSPACE_ORIGIN
            }
        }
    }

    pub(super) fn before(&self, key: &UnitKey, crates: &Crates) -> Vec<UnitKey> {
        match self {
            UnitRole::BuildRun => vec![key.sibling(UnitRole::BuildScript)],
            UnitRole::BuildScript => Self::hosted(crates.dependencies(key.package(), &Self::BUILD)),
            UnitRole::HostBuild => Self::hosted(crates.dependencies(key.package(), &Self::NORMAL))
                .into_iter()
                .chain(iter::once(key.sibling(UnitRole::BuildRun)))
                .collect(),
            UnitRole::Library => Self::checked(key, crates, &Self::NORMAL),
            UnitRole::LibraryTest => Self::checked(key, crates, &Self::NORMAL_AND_DEV),
            UnitRole::Target(_) => Self::beside_library(key, Self::checked(key, crates, &Self::NORMAL)),
            UnitRole::TestTarget(_) => Self::beside_library(key, Self::checked(key, crates, &Self::NORMAL_AND_DEV)),
        }
    }

    fn hosted(packages: Vec<&Package>) -> Vec<UnitKey> {
        packages.into_iter().map(|package| UnitKey::new(package.clone(), UnitRole::HostBuild)).collect()
    }

    fn checked(key: &UnitKey, crates: &Crates, kinds: &[Kind]) -> Vec<UnitKey> {
        crates
            .dependencies(key.package(), kinds)
            .into_iter()
            .map(|package| crates.linked(package))
            .chain(iter::once(key.sibling(UnitRole::BuildRun)))
            .collect()
    }

    fn beside_library(key: &UnitKey, checked: Vec<UnitKey>) -> Vec<UnitKey> {
        checked.into_iter().chain(iter::once(key.sibling(UnitRole::Library))).collect()
    }
}

impl fmt::Display for UnitRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnitRole::Library => formatter.write_str(" (check)"),
            UnitRole::HostBuild => formatter.write_str(" (build)"),
            UnitRole::BuildScript => formatter.write_str(" build-script"),
            UnitRole::BuildRun => formatter.write_str(" build-script (run)"),
            UnitRole::LibraryTest => formatter.write_str(" (check-test)"),
            UnitRole::Target(target) | UnitRole::TestTarget(target) => write!(formatter, "{target}"),
        }
    }
}
