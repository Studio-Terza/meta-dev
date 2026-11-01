use std::collections::{BTreeMap, BTreeSet};

use syn::visit::Visit;
use syn::{Ident, ItemUse, Macro, Path, UseTree};

use super::export::Export;
use lint_core::BinaryFiles;
use lint_core::Macros;
use lint_core::SourceFile;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Default)]
pub struct ForeignReferences {
    own_crate: Option<CrateName>,
    names: BTreeMap<CrateName, BTreeSet<Name>>,
    crates: BTreeSet<CrateName>,
    globbed: BTreeSet<CrateName>,
}

impl ForeignReferences {
    pub fn in_file(file: &SourceFile, binaries: &BinaryFiles) -> ForeignReferences {
        let own_crate = (!binaries.contains(file.path())).then(|| CrateName::from(file.placement().crate_root()));
        let mut references =
            ForeignReferences { own_crate, names: BTreeMap::new(), crates: BTreeSet::new(), globbed: BTreeSet::new() };
        references.visit_file(file.syntax());
        references
    }

    pub fn absorb(mut self, other: ForeignReferences) -> ForeignReferences {
        other.names.into_iter().for_each(|(crate_name, names)| {
            self.names.entry(crate_name).or_default().extend(names);
        });
        self.crates.extend(other.crates);
        self.globbed.extend(other.globbed);
        self
    }

    pub fn depends_on(&self, crate_name: &CrateName) -> bool {
        self.crates.contains(crate_name)
    }

    pub fn names(&self, export: &Export) -> bool {
        self.globbed.contains(export.crate_name())
            || self.names.get(export.crate_name()).is_some_and(|names| names.contains(export.name()))
    }

    fn walk(&mut self, first: Option<&Ident>, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                first
                    .is_none()
                    .then_some(&path.ident)
                    .into_iter()
                    .for_each(|crate_ident| self.record_modules(crate_ident, &path.tree));
                self.walk(Some(first.unwrap_or(&path.ident)), &path.tree);
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(first, item)),
            UseTree::Name(name) => self.record(first, &name.ident),
            UseTree::Rename(rename) => self.record(first, &rename.ident),
            UseTree::Glob(_) => self.record_glob(first),
        }
    }

    fn record_modules(&mut self, first: &Ident, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.record(Some(first), &path.ident),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.record_modules(first, item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    fn record_glob(&mut self, first: Option<&Ident>) {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .into_iter()
            .for_each(|crate_name| {
                self.crates.insert(crate_name.clone());
                self.globbed.insert(crate_name);
            });
    }

    fn record(&mut self, first: Option<&Ident>, name: &Ident) {
        first
            .map(CrateName::from)
            .filter(|crate_name| Some(crate_name) != self.own_crate.as_ref())
            .into_iter()
            .for_each(|crate_name| {
                self.names.entry(crate_name.clone()).or_default().insert(Name::from(name));
                self.crates.insert(crate_name);
            });
    }
}

impl<'ast> Visit<'ast> for ForeignReferences {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(None, &item.tree);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
        syn::visit::visit_macro(self, invocation);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        path.segments.first().zip(path.segments.iter().nth(1)).into_iter().for_each(|(first, second)| {
            self.record(Some(&first.ident), &second.ident);
        });
        syn::visit::visit_path(self, path);
    }
}
