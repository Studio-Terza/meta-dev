use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct SessionId(String);

impl TryFrom<String> for SessionId {
    type Error = Blank;

    fn try_from(text: String) -> Result<SessionId, Blank> {
        match text.chars().next() {
            Some(_) => Ok(SessionId::new(text)),
            None => Err(Blank),
        }
    }
}
