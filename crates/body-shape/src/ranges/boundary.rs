mod arithmetic;
mod classes;

use std::collections::{BTreeSet, HashMap};

use equivalence::Equivalence;
use syntax::{Lexeme, Symbol, TokenTable};

use super::Bounds;
use crate::shape::{Literals, Reading, Shape, ShapeKey};
use crate::token_count::TokenCount;
use arithmetic::{Code, Hashed, Polynomial};
use classes::{Classes, Window};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Dropped,
    Shaped(Code),
    Slotted(Symbol),
}

impl Token {
    fn counted(self) -> usize {
        match self {
            Token::Dropped => 0,
            Token::Shaped(_) | Token::Slotted(_) => 1,
        }
    }

    fn slotted(self) -> Option<Symbol> {
        match self {
            Token::Slotted(symbol) => Some(symbol),
            Token::Dropped | Token::Shaped(_) => None,
        }
    }

    fn coded(self, distance: Option<usize>) -> Option<Code> {
        match self {
            Token::Dropped => None,
            Token::Shaped(code) => Some(code),
            Token::Slotted(_) => Some(distance.map_or(Code::NEW, Code::distance)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Stretch {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub struct Shaper {
    classes: Classes,
    lexemes: Vec<Lexeme>,
    tokens: Vec<Token>,
    counted: Vec<usize>,
    codes: Vec<Code>,
    prefix: Polynomial,
    links: Links,
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
        let links = Links::linked(&tokens);
        let codes: Vec<Code> = tokens
            .iter()
            .enumerate()
            .map(|(index, token)| token.coded(links.distance(&counted, index)).unwrap_or(Code::NEW))
            .collect();
        let shaped: Vec<Code> =
            codes.iter().zip(&tokens).filter(|(_, token)| token.counted() == 1).map(|(code, _)| *code).collect();
        Shaper { classes, lexemes, prefix: Polynomial::prefix(&shaped), tokens, counted, codes, links }
    }

    pub fn read(&mut self, bounds: Bounds) -> Reading {
        let stretch = Stretch {
            start: self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.start),
            end: self.lexemes.partition_point(|lexeme| lexeme.place() < bounds.end),
        };
        self.links.advanced(stretch.start);
        let window = Window::new(stretch.start, stretch.end);
        let shape = self.stable(stretch).map_or_else(|| self.direct(window), |()| self.fast(stretch));
        Reading::new(shape, self.literals(stretch))
    }

    fn stable(&self, stretch: Stretch) -> Option<()> {
        let window = Window::new(stretch.start, stretch.end);
        let low = stretch.start..(stretch.start + Self::EDGE).min(stretch.end);
        let high = stretch.end.saturating_sub(Self::EDGE).max(stretch.start)..stretch.end;
        let moved = low
            .chain(high)
            .find(|index| self.tokens.get(*index).copied() != Some(self.classes.token(&self.lexemes, *index, window)));
        moved.map_or(Some(()), |_| None)
    }

    fn fast(&self, stretch: Stretch) -> Shape {
        let [from, to] = [stretch.start, stretch.end].map(|index| self.counted.get(index).copied().unwrap_or_default());
        let hash =
            self.links.firsts.range(stretch.start..stretch.end).fold(self.prefix.range(from, to), |hash, index| {
                let was = self.codes.get(*index).copied().unwrap_or(Code::NEW);
                let shaped = self.counted.get(*index).copied().unwrap_or_default();
                self.prefix.corrected(hash, was, to.saturating_sub(shaped + 1))
            });
        Shape::ranged(TokenCount::new(to.saturating_sub(from)), ShapeKey::new(hash.folded()))
    }

    fn direct(&self, window: Window) -> Shape {
        let tokens = window.indices().map(|index| self.classes.token(&self.lexemes, index, window));
        let walked = tokens.fold(Direct::default(), Direct::stepped);
        Shape::ranged(TokenCount::new(walked.shaped), ShapeKey::new(walked.hash.folded()))
    }

    fn literals(&self, stretch: Stretch) -> Literals {
        let window = Window::new(stretch.start, stretch.end);
        (stretch.start..stretch.end).filter_map(|index| self.classes.spelled(&self.lexemes, index, window)).collect()
    }
}

#[derive(Debug)]
struct Links {
    previous: Vec<Option<usize>>,
    next: Vec<Option<usize>>,
    firsts: BTreeSet<usize>,
    cursor: usize,
}

impl Links {
    fn linked(tokens: &[Token]) -> Links {
        let mut last = HashMap::new();
        let previous: Vec<Option<usize>> = tokens
            .iter()
            .enumerate()
            .map(|(index, token)| token.slotted().and_then(|symbol| last.insert(symbol, index)))
            .collect();
        let mut following = HashMap::new();
        let mut next: Vec<Option<usize>> = tokens
            .iter()
            .enumerate()
            .rev()
            .map(|(index, token)| token.slotted().and_then(|symbol| following.insert(symbol, index)))
            .collect();
        next.reverse();
        let firsts = previous
            .iter()
            .zip(tokens)
            .enumerate()
            .filter_map(|(index, (before, token))| token.slotted().filter(|_| before.is_none()).map(|_| index))
            .collect();
        Links { previous, next, firsts, cursor: 0 }
    }

    fn distance(&self, counted: &[usize], index: usize) -> Option<usize> {
        let before = self.previous.get(index).copied().flatten()?;
        Some(counted.get(index).copied().unwrap_or_default() - counted.get(before).copied().unwrap_or_default())
    }

    fn advanced(&mut self, start: usize) {
        let passed = self.cursor..start.max(self.cursor);
        let entering: Vec<usize> =
            passed.filter_map(|index| self.next.get(index).copied().flatten()).filter(|next| *next >= start).collect();
        self.firsts = self.firsts.split_off(&start);
        self.firsts.extend(entering);
        self.cursor = start.max(self.cursor);
    }
}

#[derive(Debug, Default)]
struct Direct {
    shaped: usize,
    seen: HashMap<Symbol, usize>,
    hash: Hashed,
}

impl Direct {
    fn stepped(mut self, token: Token) -> Direct {
        let earlier = token.slotted().and_then(|symbol| self.seen.insert(symbol, self.shaped));
        let code = token.coded(earlier.map(|before| self.shaped - before));
        let hash = code.map_or(self.hash, |found| self.hash.appended(found));
        Direct { shaped: self.shaped + token.counted(), hash, ..self }
    }
}
