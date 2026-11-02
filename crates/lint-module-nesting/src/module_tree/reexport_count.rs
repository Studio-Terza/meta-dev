use syn::{Item, UseTree, Visibility};

use lint_core::Most;
use newtype::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportCount(usize);

impl ReexportCount {
    pub fn of_file(file: &syn::File) -> ReexportCount {
        ReexportCount(
            file.items
                .iter()
                .filter_map(|item| match item {
                    Item::Use(used) if matches!(used.vis, Visibility::Public(_)) => Some(Self::leaves(&used.tree)),
                    _ => None,
                })
                .sum(),
        )
    }

    pub fn beyond(self, most: Most) -> Option<ReexportCount> {
        most.allows(self.0).map_or(Some(self), |_| None)
    }

    fn leaves(tree: &UseTree) -> usize {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().map(Self::leaves).sum(),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => 1,
        }
    }
}
