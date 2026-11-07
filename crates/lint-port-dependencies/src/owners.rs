mod co_owned;
mod foreign_dependency;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::{Dependency, Name};
use lint_placement::PackageRoot;

use co_owned::CoOwned;
use foreign_dependency::ForeignDependency;

#[derive(Debug)]
pub(crate) struct Owners<'file> {
    packages: BTreeMap<&'file PackageRoot, &'file SourceFile>,
    owning: BTreeMap<&'file Name, BTreeSet<&'file PackageRoot>>,
}

impl<'file> Owners<'file> {
    pub(crate) fn declared(files: &'file [SourceFile]) -> Owners<'file> {
        let packages: BTreeMap<&PackageRoot, &SourceFile> =
            files.iter().rev().map(|file| (file.placement().package(), file)).collect();
        let owning =
            packages.keys().fold(BTreeMap::new(), |mut owning: BTreeMap<&Name, BTreeSet<&PackageRoot>>, package| {
                package.owned().as_ref().iter().for_each(|owned| {
                    owning.entry(owned).or_default().insert(*package);
                });
                owning
            });
        Owners { packages, owning }
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
        let owners = self.owning.get(dependency.as_ref())?;
        let other = owners.iter().find(|owner| **owner != package)?.crate_name();
        let site = Site::new(file.manifest_file().as_ref(), dependency.line());
        Some(owners.get(package).map_or_else(
            || Violation::new(site, ForeignDependency::new(dependency.as_ref(), other)),
            |_| Violation::new(site, CoOwned::new(dependency.as_ref(), other)),
        ))
    }
}
