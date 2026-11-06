use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct Uuid(String);

impl TryFrom<String> for Uuid {
    type Error = Blank;

    fn try_from(text: String) -> Result<Uuid, Blank> {
        match text.chars().next() {
            Some(_) => Ok(Uuid::new(text)),
            None => Err(Blank),
        }
    }
}
