use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::Lit;

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
        let text = match literal {
            Lit::Str(found) => found.value(),
            _ => String::new(),
        };
        let braces: Vec<(usize, &str)> = text.match_indices([Self::OPEN, Self::CLOSE]).collect();
        let holes = braces
            .iter()
            .zip(braces.iter().skip(1))
            .filter_map(|((open, left), (close, right))| match (*left, *right) {
                (Self::OPENING, Self::CLOSING) => text.get(open + Self::OPENING.len()..*close),
                _ => None,
            })
            .filter_map(|hole| hole.parse::<TokenStream>().ok())
            .map(|hole| TokenTree::Group(Group::new(Delimiter::Parenthesis, hole)));
        TokenTree::Group(Group::new(Delimiter::Brace, holes.collect()))
    }
}
