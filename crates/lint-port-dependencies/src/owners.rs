mod foreign_dependency;

use std::borrow::Borrow;
use std::collections::BTreeMap;

use lint_core::{Site, SourceFile, Violation};
use lint_layout::Dependency;
use workspace_metadata::CrateName as PackageName;

use foreign_dependency::ForeignDependency;

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

    pub(crate) fn foreign(
        &self,
        bypasses: &BTreeMap<&'file PackageName, BTreeMap<&'file PackageName, &'file PackageName>>,
    ) -> Vec<Violation<'file>> {
        bypasses
            .iter()
            .filter_map(|(dependent, required)| Some((*self.manifests.get(*dependent)?, required)))
            .flat_map(|(file, required)| {
                file.placement().package().dependencies().as_ref().iter().filter_map(|dependency| {
                    let named = Self::named(dependency)?;
                    let (package, owner) = required.get_key_value(&&named)?;
                    let site = Site::new(file.manifest_file().as_ref(), dependency.line());
                    Some(Violation::new(site, ForeignDependency::new(*package, *owner)))
                })
            })
            .collect()
    }

    fn named(dependency: &Dependency) -> Option<PackageName> {
        PackageName::try_from(Borrow::<str>::borrow(dependency.as_ref())).ok()
    }
}
