use proc_macro2::{Delimiter, TokenStream, TokenTree};

use lint_layout::Whole;
use lint_layout::{Casing, Length, Name, Word, words};
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Symbol(char);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ShapeToken {
    Literal,
    Word(Name),
    Mark(Symbol),
}

impl ShapeToken {
    const MINUS: Symbol = Symbol::new('-');
    const LITERAL_WORDS: [Word<'static>; 2] = words!["true", "false"];

    pub(super) fn normalized(tokens: TokenStream) -> Vec<ShapeToken> {
        let shaped: Vec<ShapeToken> = tokens.into_iter().flat_map(Self::of_tree).collect();
        shaped
            .iter()
            .zip(shaped.iter().skip(1).map(Some).chain([None]))
            .filter(|(token, next)| !(**token == ShapeToken::Mark(Self::MINUS) && *next == Some(&ShapeToken::Literal)))
            .map(|(token, _)| token.clone())
            .collect()
    }

    fn of_tree(tree: TokenTree) -> Vec<ShapeToken> {
        match tree {
            TokenTree::Literal(_) => vec![ShapeToken::Literal],
            TokenTree::Punct(punct) => vec![ShapeToken::Mark(Symbol::new(punct.as_char()))],
            TokenTree::Ident(ident) => vec![Self::of_word(Name::from(&ident))],
            TokenTree::Group(group) => {
                let inner = Self::normalized(group.stream());
                match group.delimiter() {
                    Delimiter::Parenthesis => Self::between(Symbol::new('('), inner, Symbol::new(')')),
                    Delimiter::Brace => Self::between(Symbol::new('{'), inner, Symbol::new('}')),
                    Delimiter::Bracket => Self::between(Symbol::new('['), inner, Symbol::new(']')),
                    Delimiter::None => inner,
                }
            }
        }
    }

    fn of_word(word: Name) -> ShapeToken {
        match (word.matches::<Whole>(&Self::LITERAL_WORDS), word.casing()) {
            (Some(_), _) => ShapeToken::Literal,
            (None, Casing::Shouting) if word.length() > Length::new(1) => ShapeToken::Literal,
            _ => ShapeToken::Word(word),
        }
    }

    fn between(open: Symbol, inner: Vec<ShapeToken>, close: Symbol) -> Vec<ShapeToken> {
        [ShapeToken::Mark(open)].into_iter().chain(inner).chain([ShapeToken::Mark(close)]).collect()
    }
}
