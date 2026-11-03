use std::iter;

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::Expr;

#[derive(Debug)]
pub struct TableRows;

impl TableRows {
    const SEPARATOR: char = ',';
    const ARROW_START: char = '=';
    const ARROW_END: char = '>';
    const ARROW_LENGTH: usize = 2;

    pub fn expected(tokens: &TokenStream) -> Vec<Expr> {
        let rows = tokens.clone().into_iter().filter_map(|tree| match tree {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => Some(group.stream()),
            _ => None,
        });
        rows.last().map(|rows| Self::split(&rows.into_iter().collect::<Vec<TokenTree>>())).unwrap_or_default()
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
        let arrow = row.iter().zip(row.iter().skip(1)).position(|(start, end)| Self::arrow(start, end))?;
        syn::parse2(row.iter().skip(arrow.checked_add(Self::ARROW_LENGTH)?).cloned().collect()).ok()
    }

    fn arrow(start: &TokenTree, end: &TokenTree) -> bool {
        match (start, end) {
            (TokenTree::Punct(start), TokenTree::Punct(end)) => {
                start.as_char() == Self::ARROW_START
                    && start.spacing() == Spacing::Joint
                    && end.as_char() == Self::ARROW_END
            }
            _ => false,
        }
    }
}
