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
        Some(text).filter(|written| written.chars().all(|letter| letter != '\0')).map(Body::new).ok_or(Nul)
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
