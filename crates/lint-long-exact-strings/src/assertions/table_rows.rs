use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::Expr;

#[derive(Debug)]
pub struct TableRows;

impl TableRows {
    const SEPARATOR: char = ',';
    const ARROW_START: char = '=';
    const ARROW_END: char = '>';

    pub fn expected(tokens: &TokenStream) -> Vec<Expr> {
        let rows = tokens.clone().into_iter().filter_map(|tree| match tree {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Bracket => Some(group.stream()),
            _ => None,
        });
        rows.last().map(|rows| Self::split(rows).into_iter().filter_map(Self::wanted).collect()).unwrap_or_default()
    }

    fn split(rows: TokenStream) -> Vec<Vec<TokenTree>> {
        rows.into_iter().fold(vec![Vec::new()], |mut split, tree| {
            match &tree {
                TokenTree::Punct(punct) if punct.as_char() == Self::SEPARATOR => split.push(Vec::new()),
                _ => split.last_mut().into_iter().for_each(|row| row.push(tree.clone())),
            }
            split
        })
    }

    fn wanted(row: Vec<TokenTree>) -> Option<Expr> {
        let arrow = row.iter().zip(row.iter().skip(1)).position(|pair| match pair {
            (TokenTree::Punct(start), TokenTree::Punct(end)) => {
                start.as_char() == Self::ARROW_START
                    && start.spacing() == Spacing::Joint
                    && end.as_char() == Self::ARROW_END
            }
            _ => false,
        })?;
        syn::parse2(row.into_iter().skip(arrow.checked_add(2)?).collect()).ok()
    }
}
