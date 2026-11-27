mod boundary;
mod package;
mod unit;

use std::collections::BTreeSet;
use std::ffi::OsString;

use serde::Deserialize;

use super::name::PackageId;
use super::scope::Scope;
use crate::cli::command::failure::Failure;
pub(super) use package::Package;
use unit::Unit;

#[derive(Debug, Deserialize)]
pub(super) struct Workspace {
    packages: Vec<Package>,
    #[serde(default)]
    workspace_default_members: Vec<PackageId>,
}

impl Workspace {
    const TESTS: &'static str = "--tests";

    pub(super) fn targets(&self, scope: &Scope) -> Result<Vec<OsString>, Failure<'static>> {
        let units = scope
            .among(&self.packages, &self.workspace_default_members)?
            .into_iter()
            .flat_map(Package::units)
            .collect::<BTreeSet<Unit<'_>>>();
        units.first().ok_or(Failure::NothingToCheck)?;
        Ok(units.into_iter().flat_map(Unit::flags).chain([OsString::from(Self::TESTS)]).collect())
    }
}
