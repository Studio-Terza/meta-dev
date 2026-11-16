mod classes;

use std::borrow::Cow;
use std::collections::HashMap;

use equivalence::Equivalence;
use syntax::{Lexeme, Symbol, TokenTable};

use super::Bounds;
use crate::shape::{Code, Labels, Literals, Reading};
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

    fn coded(self, slot: Option<usize>) -> Option<Code> {
        match self {
            Token::Dropped => None,
            Token::Shaped(code) => Some(code),
            Token::Slotted(_) => slot.map(Code::Slot),
        }
    }
}

#[derive(Debug)]
pub struct Erasure<'table> {
    classes: Classes,
    lexemes: Cow<'table, [Lexeme]>,
}

impl<'table> Erasure<'table> {
    #[must_use]
    pub fn reading(table: &'table TokenTable, equivalence: Equivalence) -> Erasure<'table> {
        Erasure { classes: Classes::read(table, equivalence), lexemes: Cow::Borrowed(table.lexemes()) }
    }

    #[must_use]
    pub fn read(&self, bounds: Bounds) -> Reading {
        let window = self.window(bounds);
        let literals: Literals =
            window.indices().filter_map(|index| self.classes.spelled(&self.lexemes, index, window)).collect();
        Reading::new(literals, self.direct(window).labels)
    }

    fn window(&self, bounds: Bounds) -> Window {
        Window::new(
            self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.start),
            self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.end),
        )
    }

    fn direct(&self, window: Window) -> Direct {
        window
            .indices()
            .map(|index| self.classes.token(&self.lexemes, index, window))
            .fold(Direct::default(), Direct::stepped)
    }
}

#[derive(Debug)]
pub struct Shaper {
    erasure: Erasure<'static>,
    counted: Vec<usize>,
}

impl Shaper {
    #[must_use]
    pub fn reading(table: &TokenTable, equivalence: Equivalence) -> Shaper {
        let erasure =
            Erasure { classes: Classes::read(table, equivalence), lexemes: Cow::Owned(table.lexemes().to_vec()) };
        let whole = Window::new(0, erasure.lexemes.len());
        let tokens = whole.indices().map(|index| erasure.classes.token(&erasure.lexemes, index, whole));
        let counted = tokens.fold(vec![0], |mut counts, token| {
            counts.push(counts.last().copied().unwrap_or_default() + token.counted());
            counts
        });
        Shaper { erasure, counted }
    }

    #[must_use]
    pub fn erasure(&self) -> &Erasure<'static> {
        &self.erasure
    }

    #[must_use]
    pub fn sized(&self, bounds: Bounds) -> TokenCount {
        let window = self.erasure.window(bounds);
        let counted = |index: usize| self.counted.get(index).copied().unwrap_or_default();
        let [from, to] = window.ends().map(counted);
        TokenCount::new(to.saturating_sub(from))
    }
}

#[derive(Debug, Default)]
struct Direct {
    seen: HashMap<Symbol, usize>,
    labels: Labels,
}

impl Direct {
    fn stepped(mut self, token: Token) -> Direct {
        let next = self.seen.len();
        let slot = token.slotted().map(|symbol| *self.seen.entry(symbol).or_insert(next));
        let code = token.coded(slot);
        Direct { labels: code.into_iter().fold(self.labels, Labels::pushed), ..self }
    }
}
