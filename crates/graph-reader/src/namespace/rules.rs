use std::collections::BTreeSet;
use std::iter;
use std::sync::Arc;

use graph::Unresolved;
use lint_core::SourceFile;
use lint_layout::Name;
use syntax::{Item, ItemMacro, ItemMod, Macro, Stmt, StmtMacro, TokenTree};

use super::entries::Entries;

#[derive(Debug, Default)]
pub(crate) struct Rules(BTreeSet<Name>);

impl Rules {
    pub(crate) fn read(files: &[SourceFile]) -> Arc<Rules> {
        let sources = files.iter().flat_map(|file| iter::once(file).chain(file.test_modules()));
        Arc::new(Rules(sources.flat_map(|source| Definitions::in_items(&source.syntax().items)).collect()))
    }
}

#[derive(Debug)]
struct Definitions;

impl Definitions {
    fn in_items(items: &[Item]) -> Vec<Name> {
        items.iter().flat_map(Definitions::in_item).collect()
    }

    fn in_item(item: &Item) -> Vec<Name> {
        match item {
            Item::Macro(ItemMacro { ident: Some(ident), .. }) => Vec::from([Name::from(ident)]),
            Item::Mod(module) => Definitions::in_module(module),
            _ => Vec::new(),
        }
    }

    fn in_module(module: &ItemMod) -> Vec<Name> {
        module.content.iter().flat_map(|(_, items)| Definitions::in_items(items)).collect()
    }
}

impl Rules {
    pub(crate) fn item_entries(&self, item: &Item) -> Entries {
        match item {
            Item::Macro(ItemMacro { ident: None, mac, .. }) => self.invoked(mac),
            _ => Entries::from(item),
        }
    }

    pub(crate) fn statement_entries(&self, statement: &Stmt) -> Entries {
        match statement {
            Stmt::Item(item) => self.item_entries(item),
            Stmt::Macro(StmtMacro { mac, .. }) => self.invoked(mac),
            Stmt::Local(_) | Stmt::Expr(..) => Entries::default(),
        }
    }

    fn invoked(&self, invoked: &Macro) -> Entries {
        match self.rule(invoked) {
            Some(_) => Entries::scoped(invoked),
            None => Entries::from(invoked),
        }
    }

    fn rule(&self, invoked: &Macro) -> Option<&Name> {
        let last = invoked.path.segments.last().map(|segment| Name::from(&segment.ident))?;
        self.0.get(&last)
    }
}

#[derive(Debug)]
pub(crate) struct TokenNames;

impl TokenNames {
    pub(crate) fn named(trees: impl Iterator<Item = TokenTree>) -> BTreeSet<Name> {
        trees.flat_map(TokenNames::tree_names).collect()
    }

    fn tree_names(tree: TokenTree) -> BTreeSet<Name> {
        match tree {
            TokenTree::Ident(ident) => BTreeSet::from([Name::from(&ident)]),
            TokenTree::Group(group) => TokenNames::named(group.stream().into_iter()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => BTreeSet::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Written(BTreeSet<Name>);

impl From<&Macro> for Written {
    fn from(invoked: &Macro) -> Written {
        Written(TokenNames::named(invoked.tokens.clone().into_iter()))
    }
}

impl Written {
    pub(crate) fn names(&self) -> &BTreeSet<Name> {
        &self.0
    }

    pub(crate) fn unresolved(&self, name: &Name) -> Unresolved {
        match self.0.get(name) {
            Some(_) => Unresolved::PossiblyMacroDeclared,
            None => Unresolved::Unbound,
        }
    }
}
