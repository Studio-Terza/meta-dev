use syntax::{UseLeaf, UseTree};

use crate::name::Name;
use crate::use_path::UsePath;

#[derive(Debug, Default)]
pub struct UseEdges;

impl UseEdges {
    #[must_use]
    pub fn of_tree(tree: &UseTree) -> Vec<UsePath> {
        UseLeaf::of_tree(tree).iter().map(|leaf| leaf.path().iter().map(|ident| Name::from(*ident)).collect()).collect()
    }
}
