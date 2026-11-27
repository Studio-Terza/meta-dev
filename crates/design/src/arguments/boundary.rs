use std::ffi::OsString;
use std::vec;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(from = "Vec<String>")]
pub struct Arguments(Vec<OsString>);

impl From<Vec<String>> for Arguments {
    fn from(words: Vec<String>) -> Arguments {
        Arguments(words.into_iter().map(OsString::from).collect())
    }
}

impl From<Vec<OsString>> for Arguments {
    fn from(words: Vec<OsString>) -> Arguments {
        Arguments(words)
    }
}

impl IntoIterator for Arguments {
    type Item = OsString;
    type IntoIter = vec::IntoIter<OsString>;

    fn into_iter(self) -> vec::IntoIter<OsString> {
        self.0.into_iter()
    }
}
