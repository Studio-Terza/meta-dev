use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct Uuid(String);

impl TryFrom<String> for Uuid {
    type Error = Blank;

    fn try_from(text: String) -> Result<Uuid, Blank> {
        Some(text).filter(|named| named.chars().next().is_some()).map(Uuid::new).ok_or(Blank)
    }
}

impl AsRef<str> for Uuid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
