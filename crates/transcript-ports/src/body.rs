use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Debug, thiserror::Error)]
#[error("message text holds a NUL character")]
pub struct Nul;

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked)]
pub struct Body(String);

impl Checked for Body {
    type Error = Nul;

    fn checked(text: &String) -> Result<&String, Nul> {
        match text.chars().find(|letter| *letter == '\0') {
            Some(_) => Err(Nul),
            None => Ok(text),
        }
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
