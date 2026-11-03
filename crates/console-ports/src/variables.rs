use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};

use crate::Environment;

#[derive(Debug, Default)]
pub struct Variables(BTreeMap<OsString, OsString>);

impl FromIterator<(OsString, OsString)> for Variables {
    fn from_iter<Pairs: IntoIterator<Item = (OsString, OsString)>>(pairs: Pairs) -> Variables {
        Variables(pairs.into_iter().collect())
    }
}

impl Environment for Variables {
    fn variable(&self, name: &OsStr) -> Option<OsString> {
        self.0.get(name).cloned()
    }
}
