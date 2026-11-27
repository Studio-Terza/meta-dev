use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked)]
pub struct Instant(String);

impl Checked for Instant {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        text.chars().next().map(|_first| text).ok_or(Blank)
    }
}
