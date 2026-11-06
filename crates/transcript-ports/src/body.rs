use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
#[error("message text holds a NUL character")]
pub struct Nul;

#[derive(Newtype, Arbitrary, Display, Deserialize, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct Body(String);

impl TryFrom<String> for Body {
    type Error = Nul;

    fn try_from(text: String) -> Result<Body, Nul> {
        match text.chars().find(|letter| *letter == '\0') {
            Some(_) => Err(Nul),
            None => Ok(Body::new(text)),
        }
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
