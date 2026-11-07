use std::fmt;
use std::path::Path;

use lint_placement::CrateName;

use crate::end::{End, Node};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner<'found, 'file> {
    Typed(Node<'found>),
    Leaf(&'found CrateName, End<'found>, &'file Path),
}

impl<'found, 'file> Owner<'found, 'file> {
    pub(crate) fn of_end(end: End<'found>, krate: &'found CrateName, path: &'file Path) -> Self {
        end.node().map_or(Owner::Leaf(krate, end, path), |node| Owner::Typed(*node))
    }
}

impl fmt::Display for Owner<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Owner::Typed(node) => write!(formatter, "{node}"),
            Owner::Leaf(_, end, _) => write!(formatter, "{end}"),
        }
    }
}
