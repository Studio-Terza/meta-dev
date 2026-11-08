use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::Type;

use lint_layout::Name;

#[derive(Clone, Copy, Debug)]
pub(super) struct SelfType;

impl SelfType {
    pub(super) fn written(ty: &Type) -> Name {
        let pieces = Self::sequence(ty.to_token_stream());
        let text: String = pieces
            .iter()
            .enumerate()
            .flat_map(|(at, piece)| {
                [Self::gap(at.checked_sub(1).and_then(|before| pieces.get(before)), piece), piece.as_ref()]
            })
            .collect();
        Name::from(text.as_str())
    }

    fn sequence(stream: TokenStream) -> Vec<Piece> {
        stream.into_iter().flat_map(Piece::lexed).collect()
    }

    fn gap(before: Option<&Piece>, after: &Piece) -> &'static str {
        match before {
            Some(Piece::Word(_)) => after.spacing(),
            Some(Piece::Mark(_)) | None => "",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Brackets;

impl Brackets {
    fn around(delimiter: Delimiter) -> [&'static str; 2] {
        match delimiter {
            Delimiter::Parenthesis => ["(", ")"],
            Delimiter::Brace => ["{", "}"],
            Delimiter::Bracket => ["[", "]"],
            Delimiter::None => ["", ""],
        }
    }
}

#[derive(Debug)]
enum Piece {
    Word(String),
    Mark(String),
}

impl Piece {
    fn lexed(token: TokenTree) -> Vec<Piece> {
        match token {
            TokenTree::Ident(ident) => vec![Piece::Word(ident.to_string())],
            TokenTree::Literal(literal) => vec![Piece::Word(literal.to_string())],
            TokenTree::Punct(punct) => vec![Piece::Mark(punct.as_char().to_string())],
            TokenTree::Group(group) => Piece::grouped(&group),
        }
    }

    fn grouped(group: &Group) -> Vec<Piece> {
        let [opening, closing] = Brackets::around(group.delimiter());
        let inner = SelfType::sequence(group.stream());
        [vec![Piece::Mark(opening.to_owned())], inner, vec![Piece::Mark(closing.to_owned())]]
            .into_iter()
            .flatten()
            .collect()
    }

    fn spacing(&self) -> &'static str {
        match self {
            Piece::Word(_) => " ",
            Piece::Mark(_) => "",
        }
    }
}

impl AsRef<str> for Piece {
    fn as_ref(&self) -> &str {
        match self {
            Piece::Word(text) | Piece::Mark(text) => text,
        }
    }
}
