mod counted_name;
mod sqlx_name;

use std::borrow::Borrow;
use std::marker::PhantomData;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;

use lint_core::{Finding, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, Whole, Word};

pub(crate) use counted_name::CountedName;
pub(crate) use sqlx_name::SqlxName;

pub(crate) trait Reserved: Finding + Default {
    const OWNERS: [Word<'static>; 2];
    const NAMES: &'static [Word<'static>];
}

#[derive(Debug)]
pub(crate) struct ReservedNames<'file, Kind> {
    found: Vec<Violation<'file>>,
    kind: PhantomData<Kind>,
}

impl<'file, Kind: Reserved> From<&'file SourceFile> for ReservedNames<'file, Kind> {
    fn from(file: &'file SourceFile) -> ReservedNames<'file, Kind> {
        let found = Self::outside_owners(file)
            .map_or_default(|outside| Self::lines(outside.syntax().to_token_stream()))
            .into_iter()
            .map(|line| Violation::new(Site::new(file.head().path(), line), Kind::default()))
            .collect();
        ReservedNames { found, kind: PhantomData }
    }
}

impl<Kind: Reserved> ReservedNames<'_, Kind> {
    fn outside_owners(file: &SourceFile) -> Option<&SourceFile> {
        let name: &Name = file.head().placement().package().crate_name().borrow();
        name.matches::<Whole>(&Kind::OWNERS).map_or(Some(file), |_| None)
    }

    fn lines(tokens: TokenStream) -> Vec<LineNumber> {
        tokens.into_iter().flat_map(Self::tree_lines).collect()
    }

    fn tree_lines(tree: TokenTree) -> Vec<LineNumber> {
        match tree {
            TokenTree::Group(group) => Self::lines(group.stream()),
            TokenTree::Ident(ident) => Self::reserved(&ident).into_iter().collect(),
            TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }

    fn reserved(ident: &Ident) -> Option<LineNumber> {
        Name::from(ident).matches::<Whole>(Kind::NAMES).map(|_| LineNumber::from(ident.span()))
    }
}

impl<'file, Kind> From<ReservedNames<'file, Kind>> for Vec<Violation<'file>> {
    fn from(names: ReservedNames<'file, Kind>) -> Vec<Violation<'file>> {
        names.found
    }
}
