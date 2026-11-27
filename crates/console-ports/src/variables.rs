use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io;

use crate::{Environment, Heard};

#[derive(Debug, Default)]
pub struct Variables {
    named: BTreeMap<OsString, OsString>,
    heard: Heard,
}

impl Variables {
    #[must_use]
    pub fn hearing(self, heard: Heard) -> Variables {
        Variables { heard, ..self }
    }
}

impl FromIterator<(OsString, OsString)> for Variables {
    fn from_iter<Pairs: IntoIterator<Item = (OsString, OsString)>>(pairs: Pairs) -> Variables {
        Variables { named: pairs.into_iter().collect(), heard: Heard::default() }
    }
}

impl Environment for Variables {
    fn variable(&self, name: &OsStr) -> Option<OsString> {
        self.named.get(name).cloned()
    }

    fn input(&self) -> io::Result<Heard> {
        Ok(self.heard.clone())
    }
}
