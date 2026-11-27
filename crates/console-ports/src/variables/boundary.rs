use std::collections::BTreeMap;
use std::collections::btree_map::Iter;
use std::ffi::OsString;

use super::Variables;

impl From<BTreeMap<String, String>> for Variables {
    fn from(named: BTreeMap<String, String>) -> Variables {
        named.into_iter().map(|(name, value)| (OsString::from(name), OsString::from(value))).collect()
    }
}

impl Variables {
    pub fn iter(&self) -> Iter<'_, OsString, OsString> {
        self.named.iter()
    }
}

impl<'variables> IntoIterator for &'variables Variables {
    type Item = (&'variables OsString, &'variables OsString);
    type IntoIter = Iter<'variables, OsString, OsString>;

    fn into_iter(self) -> Iter<'variables, OsString, OsString> {
        self.iter()
    }
}
