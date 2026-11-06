use std::collections::BTreeSet;

use newtype::{Arbitrary, Newtype};
use toml::{Table, Value};

use crate::name::Name;

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Owned(BTreeSet<Name>);

impl Owned {
    const METADATA: [&'static str; 3] = ["package", "metadata", "port"];
    const OWNS: &'static str = "owns";
}

impl From<&Table> for Owned {
    fn from(manifest: &Table) -> Owned {
        Owned(
            Self::METADATA
                .iter()
                .try_fold(manifest, |table, key| table.get(*key).and_then(Value::as_table))
                .and_then(|port| port.get(Self::OWNS))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(Name::from)
                .collect(),
        )
    }
}

impl AsRef<BTreeSet<Name>> for Owned {
    fn as_ref(&self) -> &BTreeSet<Name> {
        &self.0
    }
}
