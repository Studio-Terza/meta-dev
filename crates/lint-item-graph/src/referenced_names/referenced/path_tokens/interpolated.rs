use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use syn::{Lit, LitStr};

#[derive(Debug)]
pub struct Interpolated;

impl Interpolated {
    const PAIR: usize = 2;
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
        let openings: Vec<usize> = text.match_indices(Self::OPENING).map(|(at, _)| at).collect();
        let holes = openings
            .chunk_by(|first, second| *second == *first + Self::OPENING.len())
            .filter(|run| run.len() % Self::PAIR == 1)
            .filter_map(|run| run.last())
            .map(|open| open + Self::OPENING.len())
            .filter_map(|start| text.get(start..).map(|rest| (rest, rest.match_indices(Self::CLOSING).next())))
            .filter_map(|(rest, close)| close.and_then(|(end, _)| rest.get(..end)))
            .filter_map(|hole| hole.parse::<TokenStream>().ok())
            .map(|hole| TokenTree::Group(Group::new(Delimiter::Parenthesis, hole)));
        TokenTree::Group(Group::new(Delimiter::Brace, holes.collect()))
    }
}
