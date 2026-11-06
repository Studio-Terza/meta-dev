use proc_macro2::{Delimiter, Ident, Punct, TokenTree};

use code_model::{Found, Implementation};

use super::MacroImpls;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mark {
    Opening,
    Closing,
    Metavariable,
    Word,
    Group,
    Other,
}

impl From<&Punct> for Mark {
    fn from(punct: &Punct) -> Mark {
        let character = punct.as_char();
        (character == MacroImpls::OPEN)
            .then_some(Mark::Opening)
            .or_else(|| (character == MacroImpls::CLOSE).then_some(Mark::Closing))
            .or_else(|| (character == MacroImpls::METAVARIABLE).then_some(Mark::Metavariable))
            .unwrap_or(Mark::Other)
    }
}

#[derive(Debug)]
pub(super) enum Lexeme {
    Word(Ident),
    Punctuation(Mark),
    Group { delimiter: Delimiter, children: Vec<Lexeme> },
    Other,
}

impl From<TokenTree> for Lexeme {
    fn from(token: TokenTree) -> Lexeme {
        match token {
            TokenTree::Ident(ident) => Lexeme::Word(ident),
            TokenTree::Punct(punct) => Lexeme::Punctuation(Mark::from(&punct)),
            TokenTree::Group(group) => Lexeme::Group {
                delimiter: group.delimiter(),
                children: group.stream().into_iter().map(Lexeme::from).collect(),
            },
            TokenTree::Literal(_) => Lexeme::Other,
        }
    }
}

impl Lexeme {
    pub(super) fn word(&self) -> Option<&Ident> {
        match self {
            Lexeme::Word(ident) => Some(ident),
            Lexeme::Punctuation(_) | Lexeme::Group { .. } | Lexeme::Other => None,
        }
    }

    pub(super) fn delimiter(&self) -> Option<Delimiter> {
        match self {
            Lexeme::Group { delimiter, .. } => Some(*delimiter),
            Lexeme::Word(_) | Lexeme::Punctuation(_) | Lexeme::Other => None,
        }
    }

    pub(super) fn mark(&self) -> Mark {
        match self {
            Lexeme::Word(_) => Mark::Word,
            Lexeme::Punctuation(mark) => *mark,
            Lexeme::Group { .. } => Mark::Group,
            Lexeme::Other => Mark::Other,
        }
    }

    pub(super) fn found(&self, rest: &[Lexeme]) -> Vec<Found<Implementation>> {
        match self {
            Lexeme::Word(ident) => Some(ident)
                .filter(|ident| **ident == MacroImpls::IMPL)
                .and_then(|ident| MacroImpls::header(rest, ident))
                .into_iter()
                .collect(),
            Lexeme::Group { children, .. } => MacroImpls::written(children),
            Lexeme::Punctuation(_) | Lexeme::Other => Vec::new(),
        }
    }
}
