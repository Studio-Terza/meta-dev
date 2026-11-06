mod counted_name;

use std::borrow::Borrow;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;

use lint_core::{Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, Whole, Word, words};

use counted_name::CountedName;

#[derive(Debug)]
pub(crate) struct CountedNames;

impl CountedNames {
    const COUNTING: [Word<'static>; 2] = words!["port_core", "port_derive"];
    const COUNTED: [Word<'static>; 2] = words!["counted_call", "counted_acquire"];

    pub(crate) fn outside_port_core(file: &SourceFile) -> Option<&SourceFile> {
        let name: &Name = file.placement().package().crate_name().borrow();
        name.matches::<Whole>(&Self::COUNTING).map_or(Some(file), |_| None)
    }

    pub(crate) fn named(file: &SourceFile) -> Vec<Violation<'_>> {
        Self::lines(file.syntax().to_token_stream())
            .into_iter()
            .map(|line| Violation::new(Site::new(file.path(), line), CountedName))
            .collect()
    }

    fn lines(tokens: TokenStream) -> Vec<LineNumber> {
        tokens.into_iter().flat_map(Self::tree_lines).collect()
    }

    fn tree_lines(tree: TokenTree) -> Vec<LineNumber> {
        match tree {
            TokenTree::Group(group) => Self::lines(group.stream()),
            TokenTree::Ident(ident) => Self::counted(&ident).into_iter().collect(),
            TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }

    fn counted(ident: &Ident) -> Option<LineNumber> {
        Name::from(ident).matches::<Whole>(&Self::COUNTED).map(|_| LineNumber::from(ident.span()))
    }
}
