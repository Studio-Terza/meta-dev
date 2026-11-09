use newtype::{Arbitrary, Newtype};
use serde::Deserialize;
use text_lines::OneLine;

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PackageName(OneLine);

#[derive(Newtype, Arbitrary, Clone, Debug, Deserialize, PartialEq, Eq)]
pub(super) struct PackageId(OneLine);

impl TryFrom<&str> for PackageName {
    type Error = <OneLine as TryFrom<String>>::Error;

    fn try_from(spelled: &str) -> Result<PackageName, Self::Error> {
        OneLine::try_from(spelled.to_owned()).map(PackageName)
    }
}
