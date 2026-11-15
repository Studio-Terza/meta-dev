mod counted_name;
mod sqlx_name;

use std::borrow::Borrow;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use lint_core::{Finding, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, Whole, Word};
use lint_sweep::{Finds, Watch};

pub(crate) use counted_name::CountedName;
pub(crate) use sqlx_name::SqlxName;

pub(crate) trait Reserved: Finding + Default {
    const OWNERS: [Word<'static>; 2];
    const NAMES: &'static [Word<'static>];
}

#[derive(Debug)]
struct Spelled {
    name: Name,
    line: LineNumber,
}

#[derive(Debug)]
pub(crate) struct ReservedNames<'file> {
    file: &'file SourceFile,
}

impl<'file> From<&'file SourceFile> for ReservedNames<'file> {
    fn from(file: &'file SourceFile) -> ReservedNames<'file> {
        ReservedNames { file }
    }
}

impl<'file> ReservedNames<'file> {
    fn spelled(&self) -> Vec<Spelled> {
        let text = self.file.text();
        let mentioned = text.mentioning(CountedName::NAMES).or_else(|| text.mentioning(SqlxName::NAMES));
        mentioned.map_or_default(|_| Self::idents(self.file.syntax().to_token_stream()))
    }

    fn idents(tokens: TokenStream) -> Vec<Spelled> {
        tokens.into_iter().flat_map(Self::tree_idents).collect()
    }

    fn tree_idents(tree: TokenTree) -> Vec<Spelled> {
        match tree {
            TokenTree::Group(group) => Self::idents(group.stream()),
            TokenTree::Ident(ident) => vec![Spelled { name: Name::from(&ident), line: LineNumber::from(ident.span()) }],
            TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }

    fn kept<Reserving: Reserved>(&self, spelled: &[Spelled]) -> Vec<Violation<'file>> {
        let path = self.file.head().path();
        let name: &Name = self.file.head().placement().package().crate_name().borrow();
        let outside = name.matches::<Whole>(&Reserving::OWNERS).map_or(Some(spelled), |_| None);
        let named =
            outside.into_iter().flatten().filter(|found| found.name.matches::<Whole>(Reserving::NAMES).is_some());
        named.map(|found| Violation::new(Site::new(path, found.line), Reserving::default())).collect()
    }
}

impl Watch<'_> for ReservedNames<'_> {}

impl<'file> Finds<'file> for ReservedNames<'file> {
    fn sorted(self: Box<Self>) -> Vec<Vec<Violation<'file>>> {
        let spelled = self.spelled();
        vec![self.kept::<CountedName>(&spelled), self.kept::<SqlxName>(&spelled)]
    }
}
