mod import;
mod imports;

use std::collections::BTreeSet;
use std::ptr;

use lint_core::Walk;
use lint_layout::Name;
use proc_macro2::{Ident, TokenTree};
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemImpl, ItemMod, ItemUse, UseTree};

use import::Import;
use imports::Imports;

#[derive(Debug, Default)]
pub struct Scope<'file> {
    local_roots: BTreeSet<Name>,
    imports: Imports<'file>,
}

impl<'file> Scope<'file> {
    const LOCAL_ROOTS: [&str; 3] = ["crate", "self", "super"];

    #[must_use]
    pub fn of_file(file: &'file syn::File) -> Scope<'file> {
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

    pub(crate) fn kept(&self, token: &TokenTree, root: &TokenTree, name: Name) -> Option<Name> {
        match root {
            TokenTree::Ident(ident) => self.kept_ident(token, root, ident, name),
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn kept_ident(&self, token: &TokenTree, root: &TokenTree, ident: &Ident, name: Name) -> Option<Name> {
        match Some(root).filter(|root| ptr::eq(*root, token)) {
            Some(_) => self.imports.local_item(&name, &self.local_roots).map(|imports| imports.original(name)),
            None => self.reaches(ident).map(|_| name),
        }
    }

    pub(crate) fn spelled(&self, token: &TokenTree, root: &TokenTree, name: Name) -> Vec<Name> {
        let imported = Some(root).filter(|root| ptr::eq(*root, token)).and_then(|_| self.imports.path_of(&name));
        imported.unwrap_or_else(|| vec![name])
    }

    fn reaches<'token>(&self, ident: &'token Ident) -> Option<&'token Ident> {
        self.local_roots
            .iter()
            .find(|root| **root == *ident)
            .map(|_| ident)
            .or_else(|| self.imports.holds(ident).and(self.imports.local_item(ident, &self.local_roots)).map(|_| ident))
    }

    fn import(&mut self, path: Vec<&'file Ident>, tree: &'file UseTree) {
        match tree {
            UseTree::Path(step) => self.import(path.into_iter().chain([&step.ident]).collect(), &step.tree),
            UseTree::Name(name) => self.imports.add(Import::named(path, &name.ident)),
            UseTree::Rename(rename) => self.imports.add(Some(Import::renamed(path, &rename.rename, &rename.ident))),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.import(path.clone(), item)),
            UseTree::Glob(_) => {}
        }
    }
}

impl<'file> Visit<'file> for Scope<'file> {
    fn visit_item_mod(&mut self, module: &'file ItemMod) {
        Walk::enter(&module.attrs, || {
            self.local_roots.insert(Name::from(&module.ident));
            visit::visit_item_mod(self, module);
        });
    }

    fn visit_item_use(&mut self, item: &'file ItemUse) {
        Walk::enter(&item.attrs, || self.import(Vec::new(), &item.tree));
    }

    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        Walk::enter(&item.attrs, || visit::visit_item_fn(self, item));
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        Walk::enter(&item.attrs, || visit::visit_item_impl(self, item));
    }
}
