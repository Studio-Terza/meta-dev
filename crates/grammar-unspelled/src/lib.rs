#![deny(clippy::wildcard_enum_match_arm)]

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("the text is not spelled as its grammar requires")]
pub struct Unspelled;
