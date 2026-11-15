mod counted_name;
mod sqlx_name;

use std::borrow::Borrow;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use lint_core::{Finding, Site, SourceFile, Violation};
use lint_file::{Lead, Leader};
use lint_layout::{LineNumber, Name, Whole, Word};
use lint_sweep::Member;

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
pub struct ReservedNames;

impl Leader for ReservedNames {
    const LEAD: Lead = Lead::Scanned(ReservedNames::sorted);
}

impl ReservedNames {
    pub(crate) const SQLX: Member = Member::new(1);

    fn sorted(file: &SourceFile) -> Vec<Vec<Violation<'_>>> {
        let text = file.text();
        let mentioned = text.mentioning(CountedName::NAMES).or_else(|| text.mentioning(SqlxName::NAMES));
        let spelled = mentioned.map_or_default(|_| Self::idents(file.syntax().to_token_stream()));
        vec![Self::kept::<CountedName>(file, &spelled), Self::kept::<SqlxName>(file, &spelled)]
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

    fn kept<'file, Reserving: Reserved>(file: &'file SourceFile, spelled: &[Spelled]) -> Vec<Violation<'file>> {
        let path = file.head().path();
        let name: &Name = file.head().placement().package().crate_name().borrow();
        let outside = name.matches::<Whole>(&Reserving::OWNERS).map_or(Some(spelled), |_| None);
        let named =
            outside.into_iter().flatten().filter(|found| found.name.matches::<Whole>(Reserving::NAMES).is_some());
        named.map(|found| Violation::new(Site::new(path, found.line), Reserving::default())).collect()
    }
}
