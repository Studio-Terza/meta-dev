use newtype::{Arbitrary, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PackageName(OneLine);

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PackageId(OneLine);

impl AsRef<str> for PackageName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TargetName(OneLine);

impl AsRef<str> for TargetName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
