use std::fmt::Display;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct Prompt(String);

impl<Shown: Display> From<&Shown> for Prompt {
    fn from(shown: &Shown) -> Prompt {
        Prompt(shown.to_string())
    }
}

impl AsRef<[u8]> for Prompt {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
