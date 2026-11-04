use std::convert::Infallible;
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryFailure {
    Role { text: String, reason: String },
    Top(String),
    Unreadable,
    Unfinished,
    Empty,
}

impl fmt::Display for QueryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryFailure::Role { text, reason } => write!(
                formatter,
                "the role `{text}` is not a Rust signature such as `fn f(&self, x: Self) -> Option<Self>`: {reason}"
            ),
            QueryFailure::Top(text) => write!(formatter, "--top takes a whole number, not `{text}`"),
            QueryFailure::Unreadable => formatter.write_str("an argument is not valid UTF-8"),
            QueryFailure::Unfinished => formatter.write_str(
                "--in, --role, --top, --defined, --uses, --users, --implementers and --cluster each take a value",
            ),
            QueryFailure::Empty => formatter
                .write_str("give search terms, a --role signature, or an item question such as --defined <Name>"),
        }
    }
}

impl Error for QueryFailure {}

impl From<Infallible> for QueryFailure {
    fn from(never: Infallible) -> QueryFailure {
        match never {}
    }
}
