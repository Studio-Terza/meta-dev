mod co_owned;
mod foreign_dependency;
mod unit;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::Dependency;
use lint_placement::PackageRoot;
use workspace_metadata::{CrateName as PackageName, Graph};

use co_owned::CoOwned;
use foreign_dependency::ForeignDependency;
use unit::Unit;

#[derive(Debug)]
pub(crate) struct Owners<'file> {
    packages: BTreeMap<&'file PackageRoot, &'file SourceFile>,
    units: Vec<Unit<'file>>,
}

impl<'file> Owners<'file> {
    pub(crate) fn declared(files: &'file [SourceFile], graph: &Graph<'file>) -> Owners<'file> {
        let packages: BTreeMap<&PackageRoot, &SourceFile> =
            files.iter().rev().map(|file| (file.placement().package(), file)).collect();
        let named: BTreeMap<PackageName, &PackageRoot> = packages
            .keys()
            .filter_map(|package| Some((PackageName::try_from(package.root().package()?.borrow()).ok()?, *package)))
            .collect();
        let required = graph.macros();
        let macros: BTreeMap<&PackageRoot, BTreeSet<&PackageRoot>> = named
            .iter()
            .filter(|(_, package)| package.owned().is_some())
            .map(|(name, package)| {
                let joined = required.get(name).into_iter().flatten().filter_map(|found| named.get(*found).copied());
                (*package, joined.collect())
            })
            .collect();
        let absorbed: BTreeSet<&PackageRoot> = macros.values().flatten().copied().collect();
        let ports: BTreeSet<&PackageRoot> = macros.keys().copied().collect();
        let units = ports.difference(&absorbed).filter_map(|port| Unit::joined(port, macros.get(port)?)).collect();
        Owners { packages, units }
    }

    pub(crate) fn foreign(&self) -> Vec<Violation<'file>> {
        self.packages
            .iter()
            .flat_map(|(package, file)| {
                package.dependencies().as_ref().iter().filter_map(|dependency| self.flagged(package, file, dependency))
            })
            .collect()
    }

    fn flagged(
        &self,
        package: &PackageRoot,
        file: &'file SourceFile,
        dependency: &'file Dependency,
    ) -> Option<Violation<'file>> {
        let owners: Vec<&Unit<'file>> =
            self.units.iter().filter(|unit| unit.owns().contains(dependency.as_ref())).collect();
        let first = owners.first()?;
        let site = Site::new(file.manifest_file().as_ref(), dependency.line());
        owners.iter().find(|unit| unit.holds(package).is_some()).map_or_else(
            || Some(Violation::new(site, ForeignDependency::new(dependency.as_ref(), first.port().crate_name()))),
            |_| {
                owners
                    .iter()
                    .find(|unit| unit.holds(package).is_none())
                    .map(|other| Violation::new(site, CoOwned::new(dependency.as_ref(), other.port().crate_name())))
            },
        )
    }
}
