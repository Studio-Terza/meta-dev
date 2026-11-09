mod breached;
mod expanded;
mod undenied;
mod unexpanded;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_layout::{Name, UsePath};
use lint_paths::Imports;
use lint_placement::{CrateName as PlacedName, Role as Placed};
use workspace_metadata::{CrateName, Expansion, Role};

use crate::owners::Owners;
use breached::{Breached, Fixed};
use expanded::Expanded;
use undenied::Undenied;
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

    pub(crate) fn undenied(&self, role: Role, served: &[&CrateName]) -> Vec<Violation<'file>> {
        let roots: BTreeMap<&Name, &'file SourceFile> = self
            .files
            .iter()
            .filter(|file| file.placement().role() == Placed::LibraryRoot)
            .filter_map(|file| Some((file.placement().package().root().package()?, file)))
            .collect();
        served.iter().filter_map(|name| self.denying(&roots, name, role)).flatten().collect()
    }

    fn denying(
        &self,
        roots: &BTreeMap<&Name, &'file SourceFile>,
        name: &CrateName,
        role: Role,
    ) -> Option<Vec<Violation<'file>>> {
        let (site, missing) = roots.get(&Name::from(name.as_ref())).map_or_else(
            || {
                Some((
                    Site::new(self.owners.manifest(name)?.manifest_file().as_ref(), LineNumber::FIRST),
                    Undenied::all(role),
                ))
            },
            |root| Some((Site::new(root.path(), LineNumber::FIRST), Undenied::missing(role, root))),
        )?;
        Some(missing.into_iter().map(|undenied| Violation::new(site, undenied)).collect())
    }

    pub(crate) fn unexpanded(&self, expansions: &[Expansion<'file>]) -> Vec<Violation<'file>> {
        let imports = Imports::across(self.files);
        expansions.iter().flat_map(|expansion| self.expanded_outside(expansion, &imports)).collect()
    }

    fn expanded_outside(&self, expansion: &Expansion<'file>, imports: &Imports<'_>) -> Vec<Violation<'file>> {
        let wanted: Vec<UsePath> = expansion.paths(|member, name| {
            let placed = PlacedName::from(self.owners.manifest(member)?.placement().package().root());
            let root: &Name = placed.borrow();
            Some([root.clone(), Name::from(name.as_ref())].into_iter().collect())
        });
        let admitted: BTreeSet<Name> = expansion.admitted().map(|name| Name::from(name.as_ref())).collect();
        let finding = Unexpanded::new(expansion.port(), expansion.expanders().role());
        self.files
            .iter()
            .filter_map(SourceFile::shipped)
            .filter(|file| file.placement().package().root().package().and_then(|name| admitted.get(name)).is_none())
            .flat_map(|file| {
                let lines = Expanded::lines(file, imports, &wanted);
                lines.into_iter().map(move |line| Violation::new(Site::new(file.path(), line), finding))
            })
            .collect()
    }
}
