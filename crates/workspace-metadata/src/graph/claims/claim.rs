use crate::metadata::Package;
use crate::package_id::PackageId;

use super::super::unit::Unit;

#[derive(Debug)]
pub(super) struct Claim<'unit, 'meta> {
    package: &'meta PackageId,
    open: Option<&'meta PackageId>,
    unit: &'unit Unit<'meta>,
}

impl<'unit, 'meta> Claim<'unit, 'meta> {
    pub(super) fn named(
        package: &'meta PackageId,
        owner: &'meta PackageId,
        unit: &'unit Unit<'meta>,
    ) -> Claim<'unit, 'meta> {
        Claim { package, open: Some(owner), unit }
    }

    pub(super) fn reached(package: &'meta PackageId, named: &Package, unit: &'unit Unit<'meta>) -> Claim<'unit, 'meta> {
        let owner = unit.port();
        Claim { package, open: unit.shares(named.name()).map_or(Some(owner), |_| None), unit }
    }

    pub(super) fn package(&self) -> &'meta PackageId {
        self.package
    }

    pub(super) fn open(&self) -> Option<&'meta PackageId> {
        self.open
    }

    pub(super) fn unit(&self) -> &'unit Unit<'meta> {
        self.unit
    }
}
