use lint_core::Placeholders;
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::{Lit, LitStr};

#[derive(Debug)]
pub struct Interpolated;

impl Interpolated {
    pub fn of_macro(group: &Group) -> TokenTree {
        let tokens = group.stream().into_iter().map(Self::read).collect();
        TokenTree::Group(Group::new(group.delimiter(), tokens))
    }

    fn read(token: TokenTree) -> TokenTree {
        match token {
            TokenTree::Group(group) => Self::of_macro(&group),
            TokenTree::Literal(literal) => Self::holes(Lit::new(literal)),
            TokenTree::Ident(_) | TokenTree::Punct(_) => token,
        }
    }

    fn holes(literal: Lit) -> TokenTree {
        match literal {
            Lit::Str(found) => Self::braced(&found),
            _ => TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
        }
    }

    fn braced(found: &LitStr) -> TokenTree {
        let holes = found
            .value()
            .parse::<Placeholders>()
            .into_iter()
            .flat_map(Placeholders::streams)
            .map(|hole| TokenTree::Group(Group::new(Delimiter::Parenthesis, hole)));
        TokenTree::Group(Group::new(Delimiter::Brace, holes.collect()))
    }
}
