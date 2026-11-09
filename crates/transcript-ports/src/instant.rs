use std::str::FromStr;

use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype, Spelled};

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, Spelled, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked)]
pub struct Instant(String);

impl Checked for Instant {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        match text.chars().next() {
            Some(_) => Ok(text),
            None => Err(Blank),
        }
    }
}

impl FromStr for Instant {
    type Err = Blank;

    fn from_str(text: &str) -> Result<Instant, Blank> {
        Instant::try_new(text.to_owned())
    }
}
