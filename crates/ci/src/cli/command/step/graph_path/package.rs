use std::fmt;

use text_lines::OneLine;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Package {
    name: OneLine,
    version: OneLine,
}

impl Package {
    pub(super) fn new(name: OneLine, version: OneLine) -> Package {
        Package { name, version }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
