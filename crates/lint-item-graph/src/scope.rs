mod import;
mod imports;
mod path_tokens;

use std::collections::BTreeSet;
use std::ptr;

use lint_core::Walk;
use lint_layout::{Name, UsePath};
use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemImpl, ItemMod, ItemUse, UseTree};

use import::Import;
use imports::Imports;
use path_tokens::PathTokens;

#[derive(Debug, Default)]
pub struct Scope<'file> {
    local_roots: BTreeSet<Name>,
    imports: Imports<'file>,
}

impl<'file> Scope<'file> {
    const LOCAL_ROOTS: [&str; 3] = ["crate", "self", "super"];
    const SELF_IMPORT: &str = "self";

    #[must_use]
    pub fn of_file(file: &'file syn::File) -> Scope<'file> {
        let mut scope = Scope::default();
        scope.local_roots.extend(Self::LOCAL_ROOTS.map(Name::from));
        scope.visit_file(file);
        scope
    }

    #[must_use]
    pub fn referenced(&self, tokens: TokenStream) -> Vec<UsePath> {
        let path = PathTokens::new(tokens);
        let segments: Vec<(&TokenTree, Name)> = path
            .tokens()
            .iter()
            .zip(path.roots())
            .filter_map(|(token, root)| match token {
                TokenTree::Ident(ident) => root.zip(self.kept(token, root, Name::from(ident))),
                TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
            })
            .collect();
        let nested = path.tokens().iter().flat_map(|token| match token {
            TokenTree::Group(group) => self.referenced(group.stream()),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        });
        segments
            .chunk_by(|(left, _), (right, _)| ptr::eq(*left, *right))
            .map(|path| path.iter().map(|(_, segment)| segment.clone()).collect())
            .chain(nested)
            .collect()
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

    fn kept(&self, token: &TokenTree, root: Option<&TokenTree>, name: Name) -> Option<Name> {
        match root {
            Some(root) if ptr::eq(root, token) => {
                self.imports.local_item(&name, &self.local_roots).map(|imports| imports.original(name))
            }
            Some(root) => self.local_root(Some(root)).map(|_| name),
            None => None,
        }
    }

    fn local_root<'token>(&self, token: Option<&'token TokenTree>) -> Option<&'token Ident> {
        match token? {
            TokenTree::Ident(ident) => self.reaches(ident),
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn reaches<'token>(&self, ident: &'token Ident) -> Option<&'token Ident> {
        let imported = || self.imports.holds(ident).and(self.imports.local_item(ident, &self.local_roots));
        (self.local_roots.iter().any(|root| *root == *ident) || imported().is_some()).then_some(ident)
    }

    fn import(&mut self, path: Vec<&'file Ident>, tree: &'file UseTree) {
        match tree {
            UseTree::Path(step) => self.import(path.into_iter().chain([&step.ident]).collect(), &step.tree),
            UseTree::Name(name) if name.ident == Self::SELF_IMPORT => self.imports.add(Import::itself(path)),
            UseTree::Name(name) => self.imports.add(Some(Import::renamed(path, &name.ident, &name.ident))),
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
