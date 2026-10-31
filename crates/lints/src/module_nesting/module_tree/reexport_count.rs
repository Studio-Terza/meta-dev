use std::fmt;

use syn::{Item, UseTree, Visibility};

use lint_core::Most;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReexportCount(usize);

impl ReexportCount {
    pub fn of_file(file: &syn::File) -> ReexportCount {
        ReexportCount(
            file.items
                .iter()
                .filter_map(|item| match item {
                    Item::Use(used) if !matches!(used.vis, Visibility::Inherited) => Some(Self::leaves(&used.tree)),
                    _ => None,
                })
                .sum(),
        )
    }

    pub fn exceeds(self, most: Most) -> bool {
        !most.allows(self.0)
    }

    fn leaves(tree: &UseTree) -> usize {
        match tree {
            UseTree::Path(path) => Self::leaves(&path.tree),
            UseTree::Group(group) => group.items.iter().map(Self::leaves).sum(),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => 1,
        }
    }
}

impl fmt::Display for ReexportCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
