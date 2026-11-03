use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Pair<'facts> {
    left: &'facts Name,
    right: &'facts Name,
}

impl<'facts> Pair<'facts> {
    pub(crate) fn new(left: &'facts Name, right: &'facts Name) -> Pair<'facts> {
        Pair { left, right }
    }

    pub(crate) fn witnessed_by<'mentions>(
        &self,
        mentions: &'mentions BTreeSet<Name>,
    ) -> Option<&'mentions BTreeSet<Name>> {
        (mentions.contains(self.left) && mentions.contains(self.right)).then_some(mentions)
    }
}

impl fmt::Display for Pair<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "`{}` and `{}`", self.left, self.right)
    }
}
