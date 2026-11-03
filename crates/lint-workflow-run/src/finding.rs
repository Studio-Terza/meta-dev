use std::fmt;
use std::io;

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
    Unreadable { source: io::Error },
}

impl Finding {
    const RUST_COMMAND: &'static str = "put the logic in a Rust command";
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::Block => write!(formatter, "a run step spans several lines"),
            Finding::TooLong { characters, limit } => {
                write!(formatter, "a run step is {characters} characters, over the limit of {limit}")
            }
            Finding::Operator { symbol } => write!(formatter, "a run step uses the shell operator {symbol}"),
            Finding::Control { word } => write!(formatter, "a run step starts with the shell keyword {word}"),
            Finding::Interpreter { name } => write!(formatter, "a shell setting names {name}, which is not a shell"),
            Finding::Inline { word } => write!(formatter, "a run step runs code inline through {word}"),
            Finding::Environment => write!(formatter, "an env value spans several lines, which holds logic in YAML"),
            Finding::LocalAction { target } => {
                write!(formatter, "uses {target} names no action.yml or workflow to check")
            }
            Finding::Script => write!(formatter, "a script step holds logic in YAML"),
            Finding::Unparsable { reason } => write!(formatter, "cannot parse the workflow as YAML: {reason}"),
            Finding::Unreadable { source } => write!(formatter, "cannot read the workflow: {source}"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        Fix::new(match self {
            Finding::Block
            | Finding::TooLong { .. }
            | Finding::Operator { .. }
            | Finding::Control { .. }
            | Finding::Inline { .. }
            | Finding::Environment
            | Finding::LocalAction { .. }
            | Finding::Script => Self::RUST_COMMAND,
            Finding::Interpreter { .. } => "use a shell and put the logic in a Rust command",
            Finding::Unparsable { .. } => "correct the YAML so it parses",
            Finding::Unreadable { .. } => "make the file readable as UTF-8 text",
        })
    }
}
