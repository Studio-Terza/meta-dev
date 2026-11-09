mod target;

use std::ffi::OsString;

use serde::Deserialize;

use super::super::name::{PackageId, PackageName};
use super::unit::Unit;
use crate::cli::command::failure::Failure;
use target::Target;

#[derive(Debug, Deserialize)]
pub(crate) struct Package {
    id: PackageId,
    name: PackageName,
    targets: Vec<Target>,
}

impl Package {
    pub(in super::super) fn defaulted(&self, defaults: &[PackageId]) -> Option<&Package> {
        Some(self).filter(|package| defaults.contains(&package.id))
    }

    pub(in super::super) fn resolved<'package>(
        packages: &'package [Package],
        spec: &OsString,
    ) -> Result<&'package Package, Failure<'static>> {
        packages
            .iter()
            .find(|package| spec.to_str() == Some(package.name.as_ref()))
            .ok_or_else(|| Failure::PackageSpec(spec.clone()))
    }

    pub(super) fn units(&self) -> impl Iterator<Item = Unit> {
        self.targets.iter().filter_map(Target::unit)
    }
}
