mod boundary;

use std::str::FromStr;

use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype, Spelled};

#[derive(Debug, thiserror::Error)]
#[error("message text holds a NUL character")]
pub struct Nul;

#[derive(
    Newtype, Arbitrary, Display, Spelled, CheckedDeserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord,
)]
#[newtype(checked)]
pub struct Body(String);

impl Body {
    pub(crate) fn after(self, before: &Body) -> Body {
        Body(format!("{before} {self}"))
    }
}

impl Checked for Body {
    type Error = Nul;

    fn checked(text: &String) -> Result<&String, Nul> {
        match text.chars().find(|letter| *letter == '\0') {
            Some(_) => Err(Nul),
            None => Ok(text),
        }
    }
}

impl FromStr for Body {
    type Err = Nul;

    fn from_str(text: &str) -> Result<Body, Nul> {
        Body::try_new(text.to_owned())
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
