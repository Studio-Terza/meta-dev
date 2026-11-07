use std::str::FromStr;

use newtype::{Arbitrary, Display, Newtype};

use crate::boundary::QueryFailure;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Top(usize);

impl Top {
    pub const DEFAULT: Self = Self(10);
}

impl FromStr for Top {
    type Err = QueryFailure;

    fn from_str(text: &str) -> Result<Self, QueryFailure> {
        text.parse().map(Top).map_err(|source| QueryFailure::Top { text: text.to_owned(), source })
    }
}
