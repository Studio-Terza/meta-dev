use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Pair {
    left: Name,
    right: Name,
}

impl Pair {
    pub(crate) fn new(left: Name, right: Name) -> Pair {
        Pair { left, right }
    }

    pub(crate) fn witnessed_by(&self, mentions: &BTreeSet<Name>) -> bool {
        mentions.contains(&self.left) && mentions.contains(&self.right)
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` and `{}`", self.left, self.right)
    }
}
