use proc_macro2::{Delimiter, TokenStream, TokenTree};

use lint_core::{Name, Whole};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ShapeToken {
    Literal,
    Word(Name),
    Mark(char),
}

impl ShapeToken {
    const MINUS: char = '-';
    const LITERAL_WORDS: [&str; 2] = ["true", "false"];

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
            TokenTree::Punct(punct) => vec![ShapeToken::Mark(punct.as_char())],
            TokenTree::Ident(ident) => {
                let word = Name::from(&ident);
                vec![if word.matches::<Whole>(&Self::LITERAL_WORDS) {
                    ShapeToken::Literal
                } else {
                    ShapeToken::Word(word)
                }]
            }
            TokenTree::Group(group) => {
                let inner = Self::normalized(group.stream());
                match group.delimiter() {
                    Delimiter::Parenthesis => Self::between('(', inner, ')'),
                    Delimiter::Brace => Self::between('{', inner, '}'),
                    Delimiter::Bracket => Self::between('[', inner, ']'),
                    Delimiter::None => inner,
                }
            }
        }
    }

    fn between(open: char, inner: Vec<ShapeToken>, close: char) -> Vec<ShapeToken> {
        [ShapeToken::Mark(open)].into_iter().chain(inner).chain([ShapeToken::Mark(close)]).collect()
    }
}
