use std::ffi::OsString;
use std::str::FromStr;

use design_model::Project;

use super::super::command::Hook;
use super::super::persistence::Persistence;
use crate::cli::failure::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valued {
    Consumes,
    Artifact,
    Condition,
    Reason,
}

#[derive(Clone, Copy, Debug)]
pub struct Shaped<'rest, const WORDS: usize> {
    words: [&'rest str; WORDS],
    rest: &'rest [Token],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Word(OsString),
    Persisted(Persistence),
    Valued(Valued, OsString),
    Settings,
}

impl Token {
    pub fn text(&self) -> Option<&str> {
        match self {
            Token::Word(word) => word.to_str(),
            Token::Persisted(_) | Token::Valued(..) | Token::Settings => None,
        }
    }

    pub fn persisted(&self, earlier: Persistence) -> Result<Persistence, Failure> {
        match self {
            Token::Word(_) | Token::Valued(..) | Token::Settings => Err(Failure::Usage),
            Token::Persisted(next) => earlier.then(*next),
        }
    }

    pub fn valued(&self, wanted: Valued) -> Option<&OsString> {
        match self {
            Token::Valued(valued, word) => (*valued == wanted).then_some(word),
            Token::Word(_) | Token::Persisted(_) | Token::Settings => None,
        }
    }

    pub fn hook(&self) -> Result<Hook, Failure> {
        match self {
            Token::Settings => Ok(Hook::Settings),
            Token::Word(word) => word.to_str().and_then(|event| event.parse().ok()).map(Hook::On).ok_or(Failure::Usage),
            Token::Persisted(_) | Token::Valued(..) => Err(Failure::Usage),
        }
    }

    pub fn project(&self) -> Result<Project, Failure> {
        match self {
            Token::Word(word) | Token::Valued(_, word) => Token::read(word),
            Token::Persisted(_) | Token::Settings => Err(Failure::Usage),
        }
    }

    pub fn read<Read>(word: &OsString) -> Result<Read, Failure>
    where
        Read: FromStr,
        Failure: From<Read::Err>,
    {
        Ok(word.to_str().ok_or(Failure::Usage)?.parse()?)
    }

    pub fn only(rest: &[Token]) -> Result<&str, Failure> {
        <&[Token; 1]>::try_from(rest).ok().and_then(|[word]| word.text()).ok_or(Failure::Usage)
    }

    pub fn shaped<'rest, const WORDS: usize>(
        rest: &'rest [Token],
        wanted: &[Valued],
    ) -> Result<Shaped<'rest, WORDS>, Failure> {
        let words: Vec<&str> = rest.iter().filter_map(Token::text).collect();
        let once = wanted
            .iter()
            .map(|flag| rest.iter().filter_map(|token| token.valued(*flag)).count())
            .filter(|given| *given == 1)
            .count();
        let counted = [once, WORDS + wanted.len()];
        <[&str; WORDS]>::try_from(words)
            .ok()
            .filter(|_words| counted == [wanted.len(), rest.len()])
            .map(|shaped| Shaped { words: shaped, rest })
            .ok_or(Failure::Usage)
    }
}

impl<'rest, const WORDS: usize> Shaped<'rest, WORDS> {
    pub fn words(self) -> [&'rest str; WORDS] {
        self.words
    }

    pub fn flagged(self, wanted: Valued) -> Result<&'rest OsString, Failure> {
        self.rest.iter().find_map(|token| token.valued(wanted)).ok_or(Failure::Usage)
    }
}
