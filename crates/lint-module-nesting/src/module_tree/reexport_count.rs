use std::iter::Sum;

use syn::{UseTree, Visibility};

use lint_core::Bound;
use lint_core::Syntax;
use lint_layout::Length;
use newtype::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportCount(usize);

impl ReexportCount {
    pub fn of_file(file: &syn::File) -> Self {
        file.items
            .iter()
            .filter_map(Syntax::item_use)
            .filter(|used| matches!(used.vis, Visibility::Public(_)))
            .map(|used| Self::leaves(&used.tree))
            .sum()
    }

    pub fn beyond(self, most: Bound) -> Option<Self> {
        match most.allows(Length::new(self.0)) {
            Some(_) => None,
            None => Some(self),
        }
    }

    fn leaves(tree: &UseTree) -> Self {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().map(Self::leaves).sum(),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => Self(1),
        }
    }
}

impl Sum for ReexportCount {
    fn sum<Counts: Iterator<Item = Self>>(counts: Counts) -> Self {
        Self(counts.map(|count| count.0).sum())
    }
}
