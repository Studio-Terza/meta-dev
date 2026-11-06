use std::ffi::OsString;

use design_model::Project;

use super::super::persistence::Persistence;
use crate::cli::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Word(OsString),
    Persisted(Persistence),
    Consumes(OsString),
}

impl Token {
    pub fn text(&self) -> Option<&str> {
        match self {
            Token::Word(word) => word.to_str(),
            Token::Persisted(_) | Token::Consumes(_) => None,
        }
    }

    pub fn persisted(&self, earlier: Persistence) -> Result<Persistence, Failure> {
        match self {
            Token::Word(_) | Token::Consumes(_) => Err(Failure::Usage),
            Token::Persisted(next) => earlier.then(*next),
        }
    }

    pub fn consumed(&self) -> Option<&OsString> {
        match self {
            Token::Consumes(word) => Some(word),
            Token::Word(_) | Token::Persisted(_) => None,
        }
    }

    pub fn project(&self) -> Result<Project, Failure> {
        match self {
            Token::Word(word) | Token::Consumes(word) => Ok(word.to_str().ok_or(Failure::Usage)?.parse()?),
            Token::Persisted(_) => Err(Failure::Usage),
        }
    }

    pub fn only(rest: &[Token]) -> Result<&str, Failure> {
        <&[Token; 1]>::try_from(rest).ok().and_then(|[word]| word.text()).ok_or(Failure::Usage)
    }
}
