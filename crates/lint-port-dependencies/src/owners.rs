mod foreign_dependency;
mod stale_owns;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{FileHead, Site, Violation};
use lint_layout::{LineNumber, Name, TableKey};
use workspace_metadata::{Bypass, Bypasses, CrateName as PackageName, Stale};

use foreign_dependency::ForeignDependency;
use stale_owns::StaleOwns;

#[derive(Debug)]
pub(crate) struct Owners<'file> {
    manifests: BTreeMap<Name, &'file FileHead>,
}

impl<'file> Owners<'file> {
    pub(crate) fn declared(heads: &'file [FileHead]) -> Owners<'file> {
        Owners {
            manifests: heads
                .iter()
                .rev()
                .filter_map(|head| Some((head.placement().package().root().package()?.clone(), head)))
                .collect(),
        }
    }

    pub(crate) fn manifest(&self, package: &PackageName) -> Option<&'file FileHead> {
        self.manifests.get(&Name::from(package.as_ref())).copied()
    }

    pub(crate) fn foreign(&self, bypasses: Bypasses<'file>) -> Vec<Violation<'file>> {
        bypasses
            .into_iter()
            .filter_map(|(dependent, required)| Some((*self.manifests.get(&Name::from(dependent.as_ref()))?, required)))
            .flat_map(|(file, required)| {
                required.into_iter().flat_map(move |(package, bypass)| Self::bypassing(file, package, &bypass))
            })
            .collect()
    }

    pub(crate) fn stale(&self, stale: Stale<'file>) -> Vec<Violation<'file>> {
        stale
            .into_iter()
            .filter_map(|(declarer, owned)| {
                let file = *self.manifests.get(&Name::from(declarer.as_ref()))?;
                Some((
                    Site::new(
                        file.manifest_file().as_ref(),
                        file.placement().package().key_lines().line(TableKey::PortOwns)?,
                    ),
                    owned,
                ))
            })
            .flat_map(|(site, owned)| {
                owned.into_iter().map(move |package| Violation::new(site, StaleOwns::new(package)))
            })
            .collect()
    }

    fn bypassing(file: &'file FileHead, package: &'file PackageName, bypass: &Bypass<'file>) -> Vec<Violation<'file>> {
        Self::sites(file, bypass)
            .into_iter()
            .map(|line| {
                let site = Site::new(file.manifest_file().as_ref(), line);
                Violation::new(site, ForeignDependency::new(package, bypass.owners().clone()))
            })
            .collect()
    }

    fn sites(file: &FileHead, bypass: &Bypass<'_>) -> BTreeSet<LineNumber> {
        let dependencies = file.placement().package().dependency_sites();
        Some(dependencies.sites(bypass.keys()))
            .filter(|found| found.first().is_some())
            .unwrap_or_else(|| BTreeSet::from([dependencies.header()]))
    }
}
