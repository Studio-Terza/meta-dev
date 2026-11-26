use std::iter::Sum;

use syntax::{UseLeaf, UseTree, Visibility};

use lint_core::Syntax;
use lint_layout::Length;
use newtype::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportCount(usize);

impl ReexportCount {
    pub fn of_file(file: &syntax::File) -> ReexportCount {
        file.items
            .iter()
            .filter_map(Syntax::item_use)
            .filter(|used| matches!(used.vis, Visibility::Public(_)))
            .map(|used| Self::leaves(&used.tree))
            .sum()
    }

    fn leaves(tree: &UseTree) -> ReexportCount {
        ReexportCount(UseLeaf::of_tree(tree).len())
    }
}

impl From<ReexportCount> for Length {
    fn from(count: ReexportCount) -> Length {
        Length::new(count.0)
    }
}

impl Sum for ReexportCount {
    fn sum<Counts: Iterator<Item = ReexportCount>>(counts: Counts) -> ReexportCount {
        ReexportCount(counts.map(|count| count.0).sum())
    }
}
