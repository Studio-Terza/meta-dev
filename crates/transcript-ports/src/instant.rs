use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct Instant(String);

impl TryFrom<String> for Instant {
    type Error = Blank;

    fn try_from(text: String) -> Result<Instant, Blank> {
        Some(text).filter(|named| named.chars().next().is_some()).map(Instant::new).ok_or(Blank)
    }
}
