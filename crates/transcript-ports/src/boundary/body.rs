use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Debug, thiserror::Error)]
pub enum Malformed {
    #[error("message text holds a NUL character")]
    Nul,
    #[error("message text is longer than u32::MAX bytes")]
    Oversized,
}

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked)]
pub struct Body(String);

impl Checked for Body {
    type Error = Malformed;

    fn checked(text: &String) -> Result<&String, Malformed> {
        u32::try_from(text.len()).map_err(|_oversized| Malformed::Oversized)?;
        text.chars().find(|letter| *letter == '\0').map_or(Ok(text), |_nul| Err(Malformed::Nul))
    }
}

impl Body {
    pub(crate) fn joined(&self, after: &Body) -> Result<Body, Malformed> {
        Body::try_new(format!("{self} {after}"))
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
