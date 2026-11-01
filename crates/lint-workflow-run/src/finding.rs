use std::fmt;
use std::io;

use crate::characters::Characters;

#[derive(Debug)]
pub(crate) enum Finding {
    Block,
    TooLong { characters: Characters, limit: Characters },
    Operator { symbol: &'static str },
    Control { word: &'static str },
    Unreadable { source: io::Error },
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Block => write!(formatter, "a run step spans several lines; put the logic in a Rust command"),
            Finding::TooLong { characters, limit } => {
                write!(
                    formatter,
                    "a run step is {characters} characters, over the limit of {limit}; put the logic in a Rust command"
                )
            }
            Finding::Operator { symbol } => {
                write!(formatter, "a run step uses the shell operator {symbol}; put the logic in a Rust command")
            }
            Finding::Control { word } => {
                write!(formatter, "a run step starts with the shell keyword {word}; put the logic in a Rust command")
            }
            Finding::Unreadable { source } => write!(formatter, "cannot read the workflow: {source}"),
        }
    }
}
