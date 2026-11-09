use std::str::FromStr;

use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype, Spelled};

use crate::blank::Blank;

#[derive(
    Newtype, Arbitrary, Display, Spelled, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[newtype(checked)]
pub struct SessionId(String);

impl Checked for SessionId {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        match text.chars().next() {
            Some(_) => Ok(text),
            None => Err(Blank),
        }
    }
}

impl FromStr for SessionId {
    type Err = Blank;

    fn from_str(text: &str) -> Result<SessionId, Blank> {
        SessionId::try_new(text.to_owned())
    }
}
