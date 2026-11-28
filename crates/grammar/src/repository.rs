use invariants::Ignoring;
use logos::Logos;

use crate::{Grammar, Unlexed, Whole};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Part<'text> {
    #[regex("[A-Za-z0-9_.-]+", |lexer| lexer.slice())]
    Word(&'text str),
    #[token("/")]
    Slash,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Said<'text> {
    #[regex("Spec: [^\n]+", |lexer| lexer.slice().get(6..), priority = 10, allow_greedy = true)]
    Spec(&'text str),
    #[regex("[^\n]+", allow_greedy = true)]
    Other,
    #[token("\n")]
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepositoryName<'text> {
    owner: &'text str,
    name: &'text str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecTrailer<'text>(&'text str);

impl<'text> Grammar<'text> for RepositoryName<'text> {
    fn read(text: &'text str) -> Option<RepositoryName<'text>> {
        match Whole::exactly(text)? {
            [Part::Word(owner), Part::Slash, Part::Word(name)] => Some(RepositoryName { owner, name }),
            [Part::Word(_) | Part::Slash, _, _] => None,
        }
    }
}

impl<'text> RepositoryName<'text> {
    #[must_use]
    pub fn owner(&self) -> &'text str {
        self.owner
    }

    #[must_use]
    pub fn name(&self) -> &'text str {
        self.name
    }
}

impl<'text> Grammar<'text> for SpecTrailer<'text> {
    fn read(message: &'text str) -> Option<SpecTrailer<'text>> {
        Said::lexer(message)
            .filter_map(|said| said.ignoring(Unlexed::NotThisGrammar))
            .filter_map(|said| match said {
                Said::Spec(value) => Some(SpecTrailer(value)),
                Said::Other | Said::Break => None,
            })
            .last()
    }
}

impl<'text> SpecTrailer<'text> {
    #[must_use]
    pub fn value(&self) -> &'text str {
        self.0
    }
}
