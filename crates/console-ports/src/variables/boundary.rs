use std::collections::BTreeMap;
use std::ffi::OsString;

use super::Variables;

impl From<BTreeMap<String, String>> for Variables {
    fn from(named: BTreeMap<String, String>) -> Variables {
        named.into_iter().map(|(name, value)| (OsString::from(name), OsString::from(value))).collect()
    }
}
