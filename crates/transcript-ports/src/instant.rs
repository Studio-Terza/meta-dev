use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct Instant(String);

impl TryFrom<String> for Instant {
    type Error = Blank;

    fn try_from(text: String) -> Result<Instant, Blank> {
        match text.chars().next() {
            Some(_) => Ok(Instant::new(text)),
            None => Err(Blank),
        }
    }
}
