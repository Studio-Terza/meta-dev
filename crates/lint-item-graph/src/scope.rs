mod import;
mod imports;

use std::collections::BTreeSet;

use lint_core::Walk;
use lint_layout::Name;
use proc_macro2::Ident;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemImpl, ItemMod, ItemUse, UsePath, UseTree};

use import::Import;
use imports::Imports;

#[derive(Debug, Default)]
pub struct Scope {
    local_roots: BTreeSet<Name>,
    imports: Imports,
}

impl Scope {
    const LOCAL_ROOTS: [&str; 3] = ["crate", "self", "super"];

    #[must_use]
    pub fn of_file(file: &syn::File) -> Scope {
        let mut scope = Scope::default();
        scope.local_roots.extend(Self::LOCAL_ROOTS.map(Name::from));
        scope.visit_file(file);
        scope
    }

    #[must_use]
    pub fn path_to(&self, name: &Name, written: &[Name]) -> Vec<Name> {
        match written.split_first() {
            Some((head, rest)) => self.imports.path_of(head).map_or_else(
                || written.to_vec(),
                |imported| imported.into_iter().chain(rest.iter().cloned()).collect(),
            ),
            None => self.imports.path_of(name).map_or_else(Vec::new, |mut imported| {
                imported.pop();
                imported
            }),
        }
    }

    #[must_use]
    pub fn module_root(&self, head: &Name) -> Option<&Name> {
        self.local_roots.get(head)
    }

    fn stepped(&mut self, path: &[&Ident], step: &UsePath) {
        let longer: Vec<&Ident> = path.iter().copied().chain([&step.ident]).collect();
        self.import(&longer, &step.tree);
    }

    fn import(&mut self, path: &[&Ident], tree: &UseTree) {
        match tree {
            UseTree::Path(step) => self.stepped(path, step),
            UseTree::Name(name) => self.imports.add(Import::named(path, &name.ident)),
            UseTree::Rename(rename) => self.imports.add(Some(Import::renamed(path, &rename.rename, &rename.ident))),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.import(path, item)),
            UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for Scope {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || {
            self.local_roots.insert(Name::from(&module.ident));
            visit::visit_item_mod(self, module);
        });
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        Walk::enter(&item.attrs, || self.import(&[], &item.tree));
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        Walk::enter(&item.attrs, || visit::visit_item_fn(self, item));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || visit::visit_item_impl(self, item));
    }
}
