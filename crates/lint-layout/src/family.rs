use std::collections::BTreeSet;

use newtype::{Arbitrary, Newtype};

use crate::name::Name;

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Family(BTreeSet<Name>);

impl FromIterator<Name> for Family {
    fn from_iter<Members: IntoIterator<Item = Name>>(members: Members) -> Family {
        Family(members.into_iter().collect())
    }
}

impl AsRef<BTreeSet<Name>> for Family {
    fn as_ref(&self) -> &BTreeSet<Name> {
        &self.0
    }
}
