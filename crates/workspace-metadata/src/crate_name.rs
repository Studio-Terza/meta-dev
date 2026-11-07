use newtype::{Arbitrary, Display, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateName(OneLine);

impl TryFrom<&str> for CrateName {
    type Error = <OneLine as TryFrom<String>>::Error;

    fn try_from(spelled: &str) -> Result<CrateName, Self::Error> {
        OneLine::try_from(spelled.to_owned()).map(CrateName)
    }
}

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
