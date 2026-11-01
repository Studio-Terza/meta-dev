mod import;
mod imports;
mod path_tokens;

use std::collections::BTreeSet;

use lint_core::Walk;
use lint_layout::{Name, UsePath};
use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemImpl, ItemMod, ItemUse, UseTree};

use import::Import;
use imports::Imports;
use path_tokens::PathTokens;

#[derive(Debug, Default)]
pub struct Scope {
    local_roots: BTreeSet<Name>,
    imports: Imports,
}

impl Scope {
    const LOCAL_ROOTS: [&str; 3] = ["crate", "self", "super"];
    const SELF_IMPORT: &str = "self";

    pub fn of_file(file: &syn::File) -> Scope {
        let mut scope = Scope::default();
        scope.local_roots.extend(Self::LOCAL_ROOTS.map(Name::from));
        scope.visit_file(file);
        scope
    }

    pub fn referenced(&self, tokens: TokenStream) -> Vec<Name> {
        let path = PathTokens::new(tokens);
        path.tokens()
            .iter()
            .enumerate()
            .flat_map(|(index, token)| match token {
                TokenTree::Ident(ident) => self.kept_at_index(&path, index, Name::from(ident)).into_iter().collect(),
                TokenTree::Group(group) => self.referenced(group.stream()),
                TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
            })
            .collect()
    }

    fn kept_at_index(&self, path: &PathTokens, index: usize, name: Name) -> Option<Name> {
        match path.root_index(index) {
            Some(root) if root == index => self.imports.local_item(&name, &self.local_roots),
            Some(root) => self.is_local_root(path.tokens().get(root)).then_some(name),
            None => None,
        }
    }

    fn is_local_root(&self, token: Option<&TokenTree>) -> bool {
        token.is_some_and(|token| match token {
            TokenTree::Ident(ident) => {
                let name = Name::from(ident);
                self.local_roots.contains(&name)
                    || (self.imports.holds(&name) && self.imports.local_item(&name, &self.local_roots).is_some())
            }
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => false,
        })
    }

    fn import(&mut self, prefix: &UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                self.import(&prefix.extended(Name::from(&path.ident)), &path.tree);
            }
            UseTree::Name(name) if name.ident == Self::SELF_IMPORT => {
                prefix
                    .segments()
                    .last()
                    .into_iter()
                    .for_each(|leaf| self.imports.add(Import::new(leaf.clone(), prefix.clone())));
            }
            UseTree::Name(name) => {
                self.imports.add(Import::new(Name::from(&name.ident), prefix.extended(Name::from(&name.ident))));
            }
            UseTree::Rename(rename) => {
                self.imports.add(Import::new(Name::from(&rename.rename), prefix.extended(Name::from(&rename.ident))));
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.import(prefix, item)),
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
        Walk::enter(&item.attrs, || self.import(&UsePath::default(), &item.tree));
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        Walk::enter(&item.attrs, || visit::visit_item_fn(self, item));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || visit::visit_item_impl(self, item));
    }
}
