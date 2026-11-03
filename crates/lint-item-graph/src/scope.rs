mod import;
mod imports;
mod path_tokens;

use std::collections::BTreeSet;
use std::ptr;

use lint_core::Walk;
use lint_layout::Name;
use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemImpl, ItemMod, ItemUse, UseTree};

use import::Import;
use imports::Imports;
use path_tokens::PathTokens;

#[derive(Clone, Copy, Debug)]
enum Rooting<'token> {
    Itself,
    Under(&'token TokenTree),
    Absent,
}

impl<'token> Rooting<'token> {
    fn between(token: &TokenTree, root: Option<&'token TokenTree>) -> Rooting<'token> {
        root.map_or(Rooting::Absent, |root| {
            Some(root).filter(|root| ptr::eq(*root, token)).map_or(Rooting::Under(root), |_| Rooting::Itself)
        })
    }
}

#[derive(Debug, Default)]
pub struct Scope<'file> {
    local_roots: BTreeSet<Name>,
    imports: Imports<'file>,
}

impl<'file> Scope<'file> {
    const LOCAL_ROOTS: [&str; 3] = ["crate", "self", "super"];

    pub fn of_file(file: &'file syn::File) -> Scope<'file> {
        let mut scope = Scope::default();
        scope.local_roots.extend(Self::LOCAL_ROOTS.map(Name::from));
        scope.visit_file(file);
        scope
    }

    pub fn referenced(&self, tokens: TokenStream) -> Vec<Name> {
        let path = PathTokens::new(tokens);
        path.tokens().iter().zip(path.roots()).flat_map(|(token, root)| self.token_names(token, root)).collect()
    }

    pub(crate) fn token_names(&self, token: &TokenTree, root: Option<&TokenTree>) -> Vec<Name> {
        match token {
            TokenTree::Ident(ident) => {
                self.kept(Rooting::between(token, root), Name::from(ident)).into_iter().collect()
            }
            TokenTree::Group(group) => self.referenced(group.stream()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }

    fn kept(&self, rooting: Rooting<'_>, name: Name) -> Option<Name> {
        match rooting {
            Rooting::Itself => self.imports.local_item(&name, &self.local_roots).map(|imports| imports.original(name)),
            Rooting::Under(root) => self.local_root(Some(root)).map(|_| name),
            Rooting::Absent => None,
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
        self.local_roots.iter().find(|root| **root == *ident).map(|_| ident).or_else(|| imported().map(|_| ident))
    }

    fn import(&mut self, first: Option<&'file Ident>, last: Option<&'file Ident>, tree: &'file UseTree) {
        match tree {
            UseTree::Path(path) => self.import(first.or(Some(&path.ident)), Some(&path.ident), &path.tree),
            UseTree::Name(name) => self.imports.add(Import::of_name(first, last, &name.ident)),
            UseTree::Rename(rename) => self.imports.add(Some(Import::renamed(first, &rename.rename, &rename.ident))),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.import(first, last, item)),
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
        Walk::enter(&item.attrs, || self.import(None, None, &item.tree));
    }

    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        Walk::enter(&item.attrs, || visit::visit_item_fn(self, item));
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        Walk::enter(&item.attrs, || visit::visit_item_impl(self, item));
    }
}
