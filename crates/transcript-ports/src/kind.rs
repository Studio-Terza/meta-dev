use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct Kind(String);

impl TryFrom<String> for Kind {
    type Error = Blank;

    fn try_from(text: String) -> Result<Kind, Blank> {
        match text.chars().next() {
            Some(_) => Ok(Kind::new(text)),
            None => Err(Blank),
        }
    }
}
