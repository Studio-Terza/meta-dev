use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::{Lit, LitStr};

#[derive(Debug)]
pub struct Interpolated;

impl Interpolated {
    const OPEN: char = '{';
    const CLOSE: char = '}';
    const OPENING: &str = "{";
    const CLOSING: &str = "}";

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
        let text = found.value();
        let braces: Vec<(usize, &str)> = text.match_indices([Self::OPEN, Self::CLOSE]).collect();
        let holes = braces
            .iter()
            .zip(braces.iter().skip(1))
            .filter(|((_, left), (_, right))| (*left, *right) == (Self::OPENING, Self::CLOSING))
            .filter_map(|((open, _), (close, _))| text.get(open + Self::OPENING.len()..*close))
            .filter_map(|hole| hole.parse::<TokenStream>().ok())
            .map(|hole| TokenTree::Group(Group::new(Delimiter::Parenthesis, hole)));
        TokenTree::Group(Group::new(Delimiter::Brace, holes.collect()))
    }
}
