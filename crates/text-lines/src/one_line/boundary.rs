use std::fmt;

use newtype::Checked;

#[derive(
    newtype::Newtype,
    newtype::Arbitrary,
    newtype::CheckedDeserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[newtype(checked, strategy = "\\PC*")]
pub struct OneLine(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("a line of text holds no line break or other control character")]
pub struct NotOneLine;

impl Checked for OneLine {
    type Error = NotOneLine;

    fn checked(text: &String) -> Result<&String, NotOneLine> {
        match text.chars().find(|letter| letter.is_control()) {
            Some(_) => Err(NotOneLine),
            None => Ok(text),
        }
    }
}

impl AsRef<str> for OneLine {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OneLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
