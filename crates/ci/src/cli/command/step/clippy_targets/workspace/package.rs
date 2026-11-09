mod target;

use serde::Deserialize;

use super::super::name::{PackageId, PackageName};
use super::unit::Unit;
use target::Target;

#[derive(Debug, Deserialize)]
pub(crate) struct Package {
    id: PackageId,
    name: PackageName,
    targets: Vec<Target>,
}

impl Package {
    pub(in super::super) fn id(&self) -> &PackageId {
        &self.id
    }

    pub(in super::super) fn name(&self) -> &PackageName {
        &self.name
    }

    pub(super) fn units(&self) -> impl Iterator<Item = Unit> {
        self.targets.iter().filter_map(Target::unit)
    }
}
