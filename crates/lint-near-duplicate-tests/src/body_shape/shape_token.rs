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
        let signs: Vec<Option<Symbol>> = shaped
            .iter()
            .zip(shaped.iter().skip(1).map(Some).chain([None]))
            .map(|(token, next)| {
                Some(token)
                    .filter(|token| **token == ShapeToken::Mark(Self::MINUS))
                    .filter(|_| next == Some(&ShapeToken::Literal))
                    .map(|_| Self::MINUS)
            })
            .collect();
        shaped.into_iter().zip(signs).filter(|(_, sign)| sign.is_none()).map(|(token, _)| token).collect()
    }

    fn of_tree(tree: TokenTree) -> Vec<ShapeToken> {
        match tree {
            TokenTree::Literal(_) => vec![ShapeToken::Literal],
            TokenTree::Punct(punct) => vec![ShapeToken::Mark(Symbol::new(punct.as_char()))],
            TokenTree::Ident(ident) => vec![Self::of_word(Name::from(&ident))],
            TokenTree::Group(group) => Self::grouped(group.delimiter(), Self::normalized(group.stream())),
        }
    }

    fn grouped(delimiter: Delimiter, inner: Vec<ShapeToken>) -> Vec<ShapeToken> {
        match delimiter {
            Delimiter::Parenthesis => Self::between(Symbol::new('('), inner, Symbol::new(')')),
            Delimiter::Brace => Self::between(Symbol::new('{'), inner, Symbol::new('}')),
            Delimiter::Bracket => Self::between(Symbol::new('['), inner, Symbol::new(']')),
            Delimiter::None => inner,
        }
    }

    fn of_word(word: Name) -> ShapeToken {
        let shouted = Some(word.casing())
            .filter(|casing| *casing == Casing::Shouting)
            .filter(|_| word.length() > Length::new(1))
            .map(|_| ShapeToken::Literal);
        word.matches::<Whole>(&Self::LITERAL_WORDS)
            .map(|_| ShapeToken::Literal)
            .or(shouted)
            .unwrap_or(ShapeToken::Word(word))
    }

    fn between(open: Symbol, inner: Vec<ShapeToken>, close: Symbol) -> Vec<ShapeToken> {
        [ShapeToken::Mark(open)].into_iter().chain(inner).chain([ShapeToken::Mark(close)]).collect()
    }
}
