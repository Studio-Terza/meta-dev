mod foreign_dependency;
mod stale_owns;

use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use text_lines::OneLine;
use workspace_metadata::{Bypass, Bypasses, CrateName as PackageName, Stale};

use foreign_dependency::ForeignDependency;
use stale_owns::StaleOwns;

#[derive(Debug)]
pub(crate) struct Owners<'file> {
    manifests: BTreeMap<PackageName, &'file SourceFile>,
}

impl<'file> Owners<'file> {
    pub(crate) fn declared(files: &'file [SourceFile]) -> Owners<'file> {
        Owners {
            manifests: files
                .iter()
                .rev()
                .filter_map(|file| {
                    Some((
                        PackageName::new(
                            OneLine::try_new(file.placement().package().root().package()?.to_string()).ok()?,
                        ),
                        file,
                    ))
                })
                .collect(),
        }
    }

    pub(crate) fn foreign(&self, bypasses: Bypasses<'file>) -> Vec<Violation<'file>> {
        bypasses
            .into_iter()
            .filter_map(|(dependent, required)| Some((*self.manifests.get(dependent)?, required)))
            .flat_map(|(file, required)| {
                required.into_iter().flat_map(move |(package, bypass)| Self::bypassing(file, package, &bypass))
            })
            .collect()
    }

    pub(crate) fn stale(&self, stale: Stale<'file>) -> Vec<Violation<'file>> {
        stale
            .into_iter()
            .filter_map(|(declarer, owned)| {
                let file = *self.manifests.get(declarer)?;
                Some((Site::new(file.manifest_file().as_ref(), file.placement().package().key_lines().owns()?), owned))
            })
            .flat_map(|(site, owned)| owned.into_iter().map(move |owned| Violation::new(site, StaleOwns::new(owned))))
            .collect()
    }

    fn bypassing(
        file: &'file SourceFile,
        package: &'file PackageName,
        bypass: &Bypass<'file>,
    ) -> Vec<Violation<'file>> {
        Self::sites(file, bypass)
            .into_iter()
            .map(|line| {
                let site = Site::new(file.manifest_file().as_ref(), line);
                Violation::new(site, ForeignDependency::new(package, bypass.owners().clone()))
            })
            .collect()
    }

    fn sites(file: &SourceFile, bypass: &Bypass<'_>) -> BTreeSet<LineNumber> {
        let dependencies = file.placement().package().dependency_sites();
        Some(dependencies.sites(bypass.keys()))
            .filter(|found| found.first().is_some())
            .unwrap_or_else(|| BTreeSet::from([dependencies.header()]))
    }
}
