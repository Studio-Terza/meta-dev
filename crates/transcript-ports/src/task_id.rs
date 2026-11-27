use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

use crate::blank::Blank;

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(checked)]
pub struct TaskId(String);

impl Checked for TaskId {
    type Error = Blank;

    fn checked(text: &String) -> Result<&String, Blank> {
        text.chars().next().map(|_first| text).ok_or(Blank)
    }
}

impl AsRef<str> for TaskId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
