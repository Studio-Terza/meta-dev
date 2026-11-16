use std::fmt;

use lint_layout::Name;
pub(crate) use lint_placement::Level;
use lint_placement::Nest;

pub(crate) type Cluster<'found> = Nest<'found, &'found Name>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Part<'found> {
    whole: Cluster<'found>,
    part: Cluster<'found>,
}

impl<'found> Part<'found> {
    pub(crate) fn toward(whole: Cluster<'found>, inner: Cluster<'found>) -> Part<'found> {
        Part { whole, part: inner.lineage().find(|outer| outer.parent() == Some(whole)).unwrap_or(whole) }
    }

    pub(crate) fn nested(&self) -> Option<Cluster<'found>> {
        Some(self.part).filter(|part| *part != self.whole)
    }
}

impl fmt::Display for Part<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.nested() {
            Some(part) => write!(formatter, "{part}"),
            None => write!(formatter, "{} own file", self.whole),
        }
    }
}
