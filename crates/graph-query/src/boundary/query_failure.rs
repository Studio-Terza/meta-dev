use std::convert::Infallible;
use std::error::Error;
use std::fmt;

use crate::query::{Arity, Flag};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryFailure {
    Role { text: String, reason: String },
    Top(String),
    Flag(String),
    Unreadable,
    Unfinished,
    Empty,
}

impl QueryFailure {
    fn listed(arity: Option<Arity>) -> String {
        let spellings: Vec<&str> = Flag::ALL
            .iter()
            .filter(|flag| arity.is_none_or(|arity| arity == flag.arity()))
            .map(Flag::spelling)
            .collect();
        let (init, last) = spellings.split_at(spellings.len().saturating_sub(1));
        [init.join(", "), last.join("")]
            .into_iter()
            .filter(|part| part.chars().next().is_some())
            .collect::<Vec<_>>()
            .join(" and ")
    }
}

impl fmt::Display for QueryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryFailure::Role { text, reason } => write!(
                formatter,
                "the role `{text}` is not a Rust signature such as `fn f(&self, x: Self) -> Option<Self>`: {reason}"
            ),
            QueryFailure::Top(text) => write!(formatter, "--top takes a whole number, not `{text}`"),
            QueryFailure::Flag(text) => {
                write!(formatter, "`{text}` is not a flag: the flags are {}", Self::listed(None))
            }
            QueryFailure::Unreadable => formatter.write_str("an argument is not valid UTF-8"),
            QueryFailure::Unfinished => write!(
                formatter,
                "{} each take a value, and {} takes two",
                Self::listed(Some(Arity::One)),
                Self::listed(Some(Arity::Two))
            ),
            QueryFailure::Empty => formatter.write_str(concat!(
                "give search terms, a --role signature, ",
                "or an item question such as --defined <Name>"
            )),
        }
    }
}

impl Error for QueryFailure {}

impl From<Infallible> for QueryFailure {
    fn from(never: Infallible) -> QueryFailure {
        match never {}
    }
}
