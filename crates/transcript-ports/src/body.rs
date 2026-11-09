use grammar_message::Span;
use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Debug, thiserror::Error)]
#[error("message text holds a NUL character")]
pub struct Nul;

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl Body {
    pub fn sliced(&self, span: Span) -> Result<Body, Nul> {
        Body::try_new(span.slice(&self.0).to_owned())
    }

    #[must_use]
    pub fn written(&self) -> Option<&Body> {
        self.0.chars().next().map(|_| self)
    }

    pub fn joined(&self, after: &Body) -> Result<Body, Nul> {
        Body::try_new(format!("{self} {after}"))
    }
}

impl AsRef<str> for Body {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
