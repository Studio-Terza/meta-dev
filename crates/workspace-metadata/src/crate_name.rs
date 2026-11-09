use std::str::FromStr;

use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;
use text_lines::{NotOneLine, OneLine};

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateName(OneLine);

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromStr for CrateName {
    type Err = NotOneLine;

    fn from_str(text: &str) -> Result<CrateName, NotOneLine> {
        OneLine::try_new(text.to_owned()).map(CrateName)
    }
}
