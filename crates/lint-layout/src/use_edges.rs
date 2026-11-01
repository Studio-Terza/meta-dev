use syn::visit::Visit;
use syn::{ItemUse, UseTree};

use crate::name::Name;
use crate::use_path::UsePath;

#[derive(Debug, Default)]
pub struct UseEdges(Vec<UsePath>);

impl UseEdges {
    #[must_use]
    pub fn in_file(file: &syn::File) -> Vec<UsePath> {
        let mut edges = UseEdges::default();
        edges.visit_file(file);
        edges.0
    }

    fn walk(&mut self, prefix: &UsePath, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => self.walk(&prefix.extended(Name::from(&path.ident)), &path.tree),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(prefix, item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => self.0.push(prefix.clone()),
        }
    }
}

impl<'ast> Visit<'ast> for UseEdges {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(&UsePath::default(), &item.tree);
    }
}
