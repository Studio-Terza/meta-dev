mod breached;
mod expanded;
mod unexpanded;

use std::borrow::Borrow;
use std::collections::BTreeSet;

use lint_core::{Site, SourceFile, Violation};
use lint_layout::{Name, UsePath};
use lint_paths::Imports;
use lint_placement::CrateName as PlacedName;
use text_lines::OneLine;
use workspace_metadata::{CrateName, Expansion};

use crate::owners::Owners;
use breached::{Breached, Fixed};
use expanded::Expanded;
use unexpanded::Unexpanded;

#[derive(Debug)]
pub(crate) struct Roles<'file> {
    files: &'file [SourceFile],
    owners: Owners<'file>,
}

impl<'file> Roles<'file> {
    pub(crate) fn new(files: &'file [SourceFile], owners: Owners<'file>) -> Roles<'file> {
        Roles { files, owners }
    }

    pub(crate) fn breached<'meta, Breach: Fixed<'meta>>(&self, breaches: &[Breach]) -> Vec<Violation<'file>> {
        breaches
            .iter()
            .filter_map(|breach| Some((self.owners.manifest(breach.as_ref().dependent())?, breach)))
            .flat_map(|(file, breach)| {
                let package = file.placement().package();
                let lines = Some(breach.lines(package))
                    .filter(|found| found.first().is_some())
                    .unwrap_or_else(|| BTreeSet::from([package.dependency_sites().header()]));
                let manifest = file.manifest_file().as_ref();
                lines.into_iter().map(move |line| Violation::new(Site::new(manifest, line), Breached::new(breach)))
            })
            .collect()
    }

    pub(crate) fn unexpanded(&self, expansions: &[Expansion<'file>]) -> Vec<Violation<'file>> {
        let imports = Imports::across(self.files);
        expansions.iter().flat_map(|expansion| self.expanded_outside(expansion, &imports)).collect()
    }

    fn expanded_outside(&self, expansion: &Expansion<'file>, imports: &Imports<'_>) -> Vec<Violation<'file>> {
        let port =
            self.owners.manifest(expansion.port()).map(|file| PlacedName::from(file.placement().package().root()));
        let wanted: Vec<UsePath> = port
            .iter()
            .flat_map(|placed| {
                let root: &Name = placed.borrow();
                expansion
                    .expanders()
                    .macros()
                    .iter()
                    .map(move |name| [root.clone(), Name::from(name.as_ref())].into_iter().collect())
            })
            .collect();
        let finding = Unexpanded::new(expansion.port(), expansion.expanders().role());
        self.files
            .iter()
            .filter_map(SourceFile::shipped)
            .filter(|file| Self::expander(file).and_then(|name| expansion.admits(&name)).is_none())
            .flat_map(|file| {
                let lines = Expanded::lines(file, imports, &wanted);
                lines.into_iter().map(move |line| Violation::new(Site::new(file.path(), line), finding))
            })
            .collect()
    }

    fn expander(file: &SourceFile) -> Option<CrateName> {
        OneLine::try_new(file.placement().package().root().package()?.to_string()).ok().map(CrateName::new)
    }
}
