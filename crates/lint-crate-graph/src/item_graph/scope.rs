mod import;
mod path_tokens;

use std::collections::BTreeSet;

use lint_core::{Name, UsePath};
use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{ItemMod, ItemUse, UseTree};

use import::Import;
use path_tokens::PathTokens;

#[derive(Debug, Default)]
pub struct Scope {
    local_roots: BTreeSet<Name>,
    imports: Vec<Import>,
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
        let local = match path.root_index(index) {
            Some(root) if root == index => !self.is_imported(&name),
            Some(root) => self.is_local_root(path.tokens().get(root)),
            None => false,
        };
        local.then_some(name)
    }

    fn is_local_root(&self, token: Option<&TokenTree>) -> bool {
        token.is_some_and(|token| match token {
            TokenTree::Ident(ident) => self.local_roots.contains(&Name::from(ident)),
            TokenTree::Group(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => false,
        })
    }

    fn is_imported(&self, name: &Name) -> bool {
        self.imports
            .iter()
            .filter_map(|import| import.names(name))
            .any(|path| path.first().is_some_and(|root| !self.local_roots.contains(root)))
    }

    fn import(&mut self, prefix: &UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                self.import(&prefix.extended(Name::from(&path.ident)), &path.tree);
            }
            UseTree::Name(name) if name.ident == Self::SELF_IMPORT => {
                self.imports.extend(prefix.segments().last().map(|leaf| Import::new(leaf.clone(), prefix.clone())));
            }
            UseTree::Name(name) => {
                self.imports.push(Import::new(Name::from(&name.ident), prefix.extended(Name::from(&name.ident))));
            }
            UseTree::Rename(rename) => {
                self.imports.push(Import::new(Name::from(&rename.rename), prefix.extended(Name::from(&rename.ident))));
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.import(prefix, item)),
            UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for Scope {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.local_roots.insert(Name::from(&module.ident));
        visit::visit_item_mod(self, module);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.import(&UsePath::default(), &item.tree);
    }
}
