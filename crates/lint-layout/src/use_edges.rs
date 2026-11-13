use syn::UseTree;

use crate::name::Name;
use crate::use_path::UsePath;

#[derive(Debug, Default)]
pub struct UseEdges {
    prefix: UsePath,
    found: Vec<UsePath>,
}

impl UseEdges {
    #[must_use]
    pub fn of_tree(tree: &UseTree) -> Vec<UsePath> {
        let mut edges = UseEdges::default();
        edges.walk(tree);
        edges.found
    }

    fn walk_path(&mut self, path: &syn::UsePath) {
        self.prefix.push(Name::from(&path.ident));
        self.walk(&path.tree);
        self.prefix.pop();
    }

    fn walk(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.walk_path(path),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => self.found.push(self.prefix.clone()),
        }
    }
}
