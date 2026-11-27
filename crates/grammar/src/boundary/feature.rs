use logos::Logos;

use crate::{Ignoring, Unlexed};

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = logos)]
enum Part<'text> {
    #[regex("[A-Za-z0-9_-]+", |lexer| lexer.slice())]
    Word(&'text str),
    #[token("dep:")]
    Optional,
    #[token("?")]
    Weak,
    #[token("/")]
    Through,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggled<'text> {
    Own(&'text str),
    Through { dependency: &'text str, feature: &'text str },
    Optional(&'text str),
}

impl<'text> Toggled<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<Toggled<'text>> {
        let parts = Part::lexer(text).collect::<Result<Vec<Part<'text>>, ()>>().ignoring(Unlexed::NotThisGrammar)?;
        match parts.as_slice() {
            [Part::Word(own)] => Some(Toggled::Own(own)),
            [Part::Optional, Part::Word(dependency)] => Some(Toggled::Optional(dependency)),
            [Part::Word(dependency), Part::Through, Part::Word(feature)]
            | [Part::Word(dependency), Part::Weak, Part::Through, Part::Word(feature)] => {
                Some(Toggled::Through { dependency, feature })
            }
            _ => None,
        }
    }
}
