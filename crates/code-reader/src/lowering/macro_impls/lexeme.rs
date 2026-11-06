use std::cmp::Ordering;

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};

use code_model::{Found, Implementation};

use super::super::scopes::Scopes;
use super::MacroImpls;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Mark {
    Opening,
    Closing,
    Metavariable,
    Dash,
    Word,
    Group,
    #[default]
    Other,
}

impl Mark {
    const PUNCTUATION: [(char, Mark); 4] = [
        (MacroImpls::OPEN, Mark::Opening),
        (MacroImpls::CLOSE, Mark::Closing),
        (MacroImpls::METAVARIABLE, Mark::Metavariable),
        (MacroImpls::DASH, Mark::Dash),
    ];

    fn after(self, dash: Dash) -> Mark {
        match self {
            Mark::Closing => dash.closing(),
            Mark::Opening | Mark::Metavariable | Mark::Dash | Mark::Word | Mark::Group | Mark::Other => self,
        }
    }

    fn joint(self) -> Dash {
        match self {
            Mark::Dash => Dash::Joint,
            Mark::Opening | Mark::Closing | Mark::Metavariable | Mark::Word | Mark::Group | Mark::Other => Dash::Apart,
        }
    }
}

impl From<&Punct> for Mark {
    fn from(punct: &Punct) -> Mark {
        Mark::PUNCTUATION
            .iter()
            .find(|(character, _)| *character == punct.as_char())
            .map(|(_, mark)| *mark)
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Depth {
    Outer,
    Inner,
}

impl From<Ordering> for Depth {
    fn from(ordering: Ordering) -> Depth {
        match ordering {
            Ordering::Greater => Depth::Inner,
            Ordering::Less | Ordering::Equal => Depth::Outer,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Dash {
    Joint,
    Apart,
}

impl Dash {
    fn closing(self) -> Mark {
        match self {
            Dash::Joint => Mark::Other,
            Dash::Apart => Mark::Closing,
        }
    }
}

impl From<&Punct> for Dash {
    fn from(punct: &Punct) -> Dash {
        match punct.spacing() {
            Spacing::Joint => Mark::from(punct).joint(),
            Spacing::Alone => Dash::Apart,
        }
    }
}

impl From<&TokenTree> for Dash {
    fn from(token: &TokenTree) -> Dash {
        match token {
            TokenTree::Punct(punct) => Dash::from(punct),
            TokenTree::Ident(_) | TokenTree::Group(_) | TokenTree::Literal(_) => Dash::Apart,
        }
    }
}

#[derive(Debug)]
pub(super) enum Lexeme {
    Word(Ident),
    Punctuation(Mark),
    Group { delimiter: Delimiter, children: Vec<Lexeme> },
    Other,
}

impl Lexeme {
    pub(super) fn sequence(stream: TokenStream) -> Vec<Lexeme> {
        stream
            .into_iter()
            .scan(Dash::Apart, |previous, token| {
                let dash = std::mem::replace(previous, Dash::from(&token));
                Some(Lexeme::new(token, dash))
            })
            .collect()
    }

    fn new(token: TokenTree, dash: Dash) -> Lexeme {
        match token {
            TokenTree::Ident(ident) => Lexeme::Word(ident),
            TokenTree::Punct(punct) => Lexeme::Punctuation(Mark::from(&punct).after(dash)),
            TokenTree::Group(group) => {
                Lexeme::Group { delimiter: group.delimiter(), children: Self::sequence(group.stream()) }
            }
            TokenTree::Literal(_) => Lexeme::Other,
        }
    }

    pub(super) fn depths(lexemes: &[Lexeme]) -> Vec<Depth> {
        (0..lexemes.len())
            .map(|at| {
                let before: Vec<Mark> = lexemes.get(..at).unwrap_or_default().iter().map(Lexeme::mark).collect();
                let opened = before.iter().filter(|mark| **mark == Mark::Opening).count();
                let closed = before.iter().filter(|mark| **mark == Mark::Closing).count();
                Depth::from(opened.cmp(&closed))
            })
            .collect()
    }

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

    pub(super) fn found(&self, rest: &[Lexeme], scopes: &Scopes) -> Vec<Found<Implementation>> {
        match self {
            Lexeme::Word(ident) => Some(ident)
                .filter(|ident| **ident == MacroImpls::IMPL)
                .and_then(|ident| MacroImpls::header(rest, ident, scopes))
                .into_iter()
                .collect(),
            Lexeme::Group { children, .. } => MacroImpls::written(children, scopes),
            Lexeme::Punctuation(_) | Lexeme::Other => Vec::new(),
        }
    }
}
