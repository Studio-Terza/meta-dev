use std::iter::Sum;

use syn::{Item, UseTree, Visibility};

use lint_core::Most;
use lint_layout::Length;
use newtype::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportCount(usize);

impl ReexportCount {
    pub fn of_file(file: &syn::File) -> ReexportCount {
        file.items
            .iter()
            .filter_map(|item| match item {
                Item::Use(used) if matches!(used.vis, Visibility::Public(_)) => Some(Self::leaves(&used.tree)),
                _ => None,
            })
            .sum()
    }

    pub fn beyond(self, most: Most) -> Option<ReexportCount> {
        match most.allows(Length::new(self.0)) {
            Some(_) => None,
            None => Some(self),
        }
    }

    fn leaves(tree: &UseTree) -> ReexportCount {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().map(Self::leaves).sum(),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => ReexportCount(1),
        }
    }
}

impl Sum for ReexportCount {
    fn sum<Counts: Iterator<Item = ReexportCount>>(counts: Counts) -> ReexportCount {
        ReexportCount(counts.map(|count| count.0).sum())
    }
}
