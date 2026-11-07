use std::fmt;

use lint_core::Fix;

use crate::characters::Characters;
use crate::excerpt::Excerpt;
use crate::word::Word;

#[derive(Debug)]
pub(crate) enum Finding {
    Block,
    TooLong { characters: Characters, limit: Characters },
    Operator { symbol: Word },
    Control { word: Word },
    Interpreter { name: Excerpt },
    Inline { word: Word },
    Environment,
    LocalAction { target: Excerpt },
    Script,
    Unparsable { reason: Excerpt },
    Unreadable { reason: Excerpt },
}

impl Finding {
    const RUST_COMMAND: &'static str = "put the logic in a Rust command";
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Block => write!(formatter, "a run step spans several lines"),
            Self::TooLong { characters, limit } => {
                write!(formatter, "a run step is {characters} characters, over the limit of {limit}")
            }
            Self::Operator { symbol } => write!(formatter, "a run step uses the shell operator {symbol}"),
            Self::Control { word } => write!(formatter, "a run step starts with the shell keyword {word}"),
            Self::Interpreter { name } => write!(formatter, "a shell setting names {name}, which is not a shell"),
            Self::Inline { word } => write!(formatter, "a run step runs code inline through {word}"),
            Self::Environment => write!(formatter, "an env value spans several lines, which holds logic in YAML"),
            Self::LocalAction { target } => {
                write!(formatter, "uses {target} names no action.yml or workflow to check")
            }
            Self::Script => write!(formatter, "a script step holds logic in YAML"),
            Self::Unparsable { reason } => write!(formatter, "cannot parse the workflow as YAML: {reason}"),
            Self::Unreadable { reason } => write!(formatter, "cannot read the workflow: {reason}"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        match self {
            Self::Block
            | Self::TooLong { .. }
            | Self::Operator { .. }
            | Self::Control { .. }
            | Self::Inline { .. }
            | Self::Environment
            | Self::LocalAction { .. }
            | Self::Script => Fix::new(Self::RUST_COMMAND),
            Self::Interpreter { .. } => Fix::new("use a shell and put the logic in a Rust command"),
            Self::Unparsable { .. } => Fix::new("correct the YAML so it parses"),
            Self::Unreadable { .. } => Fix::new("make the file readable as UTF-8 text"),
        }
    }
}
