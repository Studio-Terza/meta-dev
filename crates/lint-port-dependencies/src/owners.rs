mod foreign_dependency;
mod stale_owns;

use std::borrow::Borrow;
use std::collections::BTreeMap;

use lint_core::{Site, SourceFile, Violation};
use lint_layout::Dependency;
use workspace_metadata::{Bypasses, CrateName as PackageName, Stale};

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
                    Some((PackageName::try_from(file.placement().package().root().package()?.borrow()).ok()?, file))
                })
                .collect(),
        }
    }

    pub(crate) fn foreign(&self, bypasses: Bypasses<'file>) -> Vec<Violation<'file>> {
        bypasses
            .into_iter()
            .filter_map(|(dependent, required)| Some((*self.manifests.get(dependent)?, required)))
            .flat_map(|(file, required)| {
                file.placement().package().dependencies().as_ref().iter().filter_map(move |dependency| {
                    let (package, owners) = required.get_key_value(&Self::named(dependency)?)?;
                    let site = Site::new(file.manifest_file().as_ref(), dependency.line());
                    Some(Violation::new(site, ForeignDependency::new(package, owners.clone())))
                })
            })
            .collect()
    }

    pub(crate) fn stale(&self, stale: Stale<'file>) -> Vec<Violation<'file>> {
        stale
            .into_iter()
            .filter_map(|(declarer, owned)| {
                let file = *self.manifests.get(declarer)?;
                Some((Site::new(file.manifest_file().as_ref(), file.placement().package().owns()?), owned))
            })
            .flat_map(|(site, owned)| owned.into_iter().map(move |owned| Violation::new(site, StaleOwns::new(owned))))
            .collect()
    }

    fn named(dependency: &Dependency) -> Option<PackageName> {
        PackageName::try_from(Borrow::<str>::borrow(dependency.as_ref())).ok()
    }
}
