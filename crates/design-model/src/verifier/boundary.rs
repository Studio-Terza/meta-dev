use std::str::FromStr;

use grammar_project::Names;
use newtype::{Arbitrary, Checked, CheckedDeserialize, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked, strategy = "[a-z0-9][a-z0-9-]{0,15}")]
pub struct FeatureName(String);

#[derive(Newtype, Arbitrary, Display, CheckedDeserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[newtype(checked, strategy = "[a-z0-9][a-z0-9_-]{0,15}")]
pub struct CrateName(String);

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a name, which is letters, digits, '.', '_' and '-', starting with a letter or digit")]
pub struct Misnamed(String);

impl Checked for FeatureName {
    type Error = Misnamed;

    fn checked(text: &String) -> Result<&String, Misnamed> {
        Names::named(text).map(|_named| text).ok_or_else(|| Misnamed(text.clone()))
    }
}

impl Checked for CrateName {
    type Error = Misnamed;

    fn checked(text: &String) -> Result<&String, Misnamed> {
        Names::named(text).map(|_named| text).ok_or_else(|| Misnamed(text.clone()))
    }
}

impl FromStr for FeatureName {
    type Err = Misnamed;

    fn from_str(text: &str) -> Result<FeatureName, Misnamed> {
        FeatureName::try_new(text.to_owned())
    }
}

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
