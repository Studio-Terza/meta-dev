mod classes;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use equivalence::Equivalence;
use syntax::{Lexeme, Symbol, TokenTable};

use super::Bounds;
use crate::shape::{Code, Labels, Literals, Reading, Shape, ShapeKey};
use crate::token_count::TokenCount;
use classes::{Classes, Window};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Dropped,
    Shaped(Code),
    Slotted(Symbol),
}

impl Token {
    fn counted(&self) -> usize {
        match self {
            Token::Dropped => 0,
            Token::Shaped(_) | Token::Slotted(_) => 1,
        }
    }

    fn slotted(&self) -> Option<Symbol> {
        match self {
            Token::Slotted(symbol) => Some(*symbol),
            Token::Dropped | Token::Shaped(_) => None,
        }
    }

    fn coded(self, distance: Option<usize>) -> Option<Code> {
        match self {
            Token::Dropped => None,
            Token::Shaped(code) => Some(code),
            Token::Slotted(_) => Some(distance.map_or(Code::New, Code::Distance)),
        }
    }
}

#[derive(Debug)]
pub struct Shaper {
    classes: Classes,
    lexemes: Vec<Lexeme>,
    tokens: Vec<Token>,
    counted: Vec<usize>,
}

impl Shaper {
    const EDGE: usize = 2;

    #[must_use]
    pub fn reading(table: &TokenTable, equivalence: Equivalence) -> Shaper {
        let classes = Classes::read(table, equivalence);
        let lexemes = table.lexemes().to_vec();
        let whole = Window::new(0, lexemes.len());
        let tokens: Vec<Token> = (0..lexemes.len()).map(|index| classes.token(&lexemes, index, whole)).collect();
        let counted = tokens.iter().fold(vec![0], |mut counts, token| {
            counts.push(counts.last().copied().unwrap_or_default() + token.counted());
            counts
        });
        Shaper { classes, lexemes, tokens, counted }
    }

    #[must_use]
    pub fn sized(&self, bounds: Bounds) -> TokenCount {
        let window = self.window(bounds);
        let counted = |index: usize| self.counted.get(index).copied().unwrap_or_default();
        let [from, to] = window.ends().map(counted);
        self.stable(window).map_or_else(|| self.direct(window).size(), |()| TokenCount::new(to.saturating_sub(from)))
    }

    #[must_use]
    pub fn read(&self, bounds: Bounds) -> Reading {
        let window = self.window(bounds);
        let literals: Literals =
            window.indices().filter_map(|index| self.classes.spelled(&self.lexemes, index, window)).collect();
        let direct = self.direct(window);
        Reading::new(direct.shape(), literals, direct.labels)
    }

    fn window(&self, bounds: Bounds) -> Window {
        Window::new(
            self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.start),
            self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.end),
        )
    }

    fn stable(&self, window: Window) -> Option<()> {
        let [start, end] = window.ends();
        let low = start..(start + Self::EDGE).min(end);
        let high = end.saturating_sub(Self::EDGE).max(start)..end;
        let moved = low
            .chain(high)
            .find(|index| self.tokens.get(*index).cloned() != Some(self.classes.token(&self.lexemes, *index, window)));
        moved.map_or(Some(()), |_| None)
    }

    fn direct(&self, window: Window) -> Direct {
        window
            .indices()
            .map(|index| self.classes.token(&self.lexemes, index, window))
            .fold(Direct::default(), Direct::stepped)
    }
}

#[derive(Debug, Default)]
struct Direct {
    seen: HashMap<Symbol, usize>,
    labels: Labels,
}

impl Direct {
    fn stepped(mut self, token: Token) -> Direct {
        let shaped = self.labels.count();
        let earlier = token.slotted().and_then(|symbol| self.seen.insert(symbol, shaped));
        let code = token.coded(earlier.map(|before| shaped - before));
        Direct { labels: code.into_iter().fold(self.labels, Labels::pushed), ..self }
    }

    fn size(&self) -> TokenCount {
        TokenCount::new(self.labels.count())
    }

    fn shape(&self) -> Shape {
        let mut hasher = DefaultHasher::new();
        self.labels.hash(&mut hasher);
        Shape::ranged(self.size(), ShapeKey::new(hasher.finish()))
    }
}
