use std::ffi::OsString;

use super::super::persistence::Persistence;
use crate::cli::failure::Failure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Word(OsString),
    Persisted(Persistence),
}

impl Token {
    pub fn text(&self) -> Option<&str> {
        match self {
            Token::Word(word) => word.to_str(),
            Token::Persisted(_) => None,
        }
    }

    pub fn persisted(&self, earlier: Persistence) -> Result<Persistence, Failure> {
        match self {
            Token::Word(_) => Err(Failure::Usage),
            Token::Persisted(next) => earlier.then(*next),
        }
    }

    pub fn only(rest: &[Token]) -> Result<&str, Failure> {
        <&[Token; 1]>::try_from(rest).ok().and_then(|[word]| word.text()).ok_or(Failure::Usage)
    }
}
