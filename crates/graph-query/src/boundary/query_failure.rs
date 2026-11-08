use std::convert::Infallible;
use std::num::ParseIntError;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum QueryFailure {
    #[error("the role `{text}` is not a Rust signature such as `fn f(&self, x: Self) -> Option<Self>`: {reason}")]
    Role { text: String, reason: String },
    #[error("--top takes a whole number, not `{text}`")]
    Top { text: String, source: ParseIntError },
    #[error("an argument is not valid UTF-8")]
    Unreadable,
    #[error(
        "--in, --role, --top, --defined, --uses, --users, --implementers, --cluster and --exports each take a value, and --path takes two"
    )]
    Unfinished,
    #[error("give search terms, a --role signature, or an item question such as --defined <Name>")]
    Empty,
}

impl From<Infallible> for QueryFailure {
    fn from(never: Infallible) -> QueryFailure {
        match never {}
    }
}
