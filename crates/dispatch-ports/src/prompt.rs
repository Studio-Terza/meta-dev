use std::fmt::{self, Write};

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Default, PartialEq, Eq)]
pub struct Prompt(String);

impl<Shown: fmt::Display + ?Sized> From<&Shown> for Prompt {
    fn from(shown: &Shown) -> Prompt {
        let mut text = String::new();
        write!(text, "{shown}").map_or_else(|_| Prompt::default(), |()| Prompt(text))
    }
}

impl AsRef<str> for Prompt {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
