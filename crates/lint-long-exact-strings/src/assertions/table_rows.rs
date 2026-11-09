use std::iter;

use lint_core::Syntax;
use proc_macro2::{Delimiter, Group, Spacing, TokenStream, TokenTree};
use syn::Expr;

#[derive(Debug)]
pub struct TableRows;

impl TableRows {
    const SEPARATOR: char = ',';
    const ARROW_START: char = '=';
    const ARROW_END: char = '>';
    const ARROW_LENGTH: usize = 2;

    pub fn expected(tokens: &TokenStream) -> Vec<Expr> {
        let rows = tokens.clone().into_iter().filter_map(|tree| {
            Syntax::group(&tree).filter(|group| group.delimiter() == Delimiter::Bracket).map(Group::stream)
        });
        rows.last().map(|last| Self::split(&last.into_iter().collect::<Vec<TokenTree>>())).unwrap_or_default()
    }

    fn split(rows: &[TokenTree]) -> Vec<Expr> {
        let cuts: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, tree)| matches!(tree, TokenTree::Punct(punct) if punct.as_char() == Self::SEPARATOR))
            .map(|(index, _)| index)
            .collect();
        let starts = iter::once(0).chain(cuts.iter().map(|cut| cut.saturating_add(1)));
        let ends = cuts.iter().copied().chain(iter::once(rows.len()));
        starts.zip(ends).filter_map(|(start, end)| rows.get(start..end)).filter_map(Self::wanted).collect()
    }

    fn wanted(row: &[TokenTree]) -> Option<Expr> {
        let arrow = row.iter().zip(row.iter().skip(1)).position(|(start, end)| Self::arrow(start, end).is_some())?;
        syn::parse2(row.iter().skip(arrow.checked_add(Self::ARROW_LENGTH)?).cloned().collect()).ok()
    }

    fn arrow<'end>(start: &TokenTree, end: &'end TokenTree) -> Option<&'end TokenTree> {
        Syntax::punct(start)
            .filter(|first| first.as_char() == Self::ARROW_START)
            .filter(|first| first.spacing() == Spacing::Joint)
            .and(Syntax::punct(end))
            .filter(|second| second.as_char() == Self::ARROW_END)
            .map(|_| end)
    }
}
