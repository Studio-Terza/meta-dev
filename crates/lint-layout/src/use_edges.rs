use syn::visit::Visit;
use syn::{ItemUse, UseTree};

use crate::name::Name;
use crate::use_path::UsePath;

#[derive(Debug)]
pub struct UseEdges<Edge: FnMut(&UsePath)> {
    prefix: UsePath,
    edge: Edge,
}

impl<Edge: FnMut(&UsePath)> UseEdges<Edge> {
    pub fn each(file: &syn::File, edge: Edge) {
        let mut edges = UseEdges { prefix: UsePath::default(), edge };
        edges.visit_file(file);
    }

    fn walk(&mut self, tree: &UseTree) {
        match tree {
            UseTree::Path(path) => {
                self.prefix.push(Name::from(&path.ident));
                self.walk(&path.tree);
                self.prefix.pop();
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.walk(item)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => (self.edge)(&self.prefix),
        }
    }
}

impl<'ast, Edge: FnMut(&UsePath)> Visit<'ast> for UseEdges<Edge> {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.walk(&item.tree);
    }
}
