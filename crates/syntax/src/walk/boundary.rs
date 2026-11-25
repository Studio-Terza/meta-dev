use syn::visit::{self, Visit};
use syn::{File, Ident, Macro};

use super::Node;

#[derive(Clone, Copy, Debug)]
pub struct Entered<'ast> {
    node: Node<'ast>,
    depth: usize,
}

struct Preorder<'ast> {
    entered: Vec<Entered<'ast>>,
    depth: usize,
}

macro_rules! preorder {
    ($($kind:ident $on:ident $off:ident $visit:ident $node:ident;)*) => {
        impl<'ast> Visit<'ast> for Preorder<'ast> {
            fn visit_macro(&mut self, node: &'ast Macro) {
                let outer = self.depth;
                self.entered.push(Entered { node: Node::Macro(node), depth: outer });
                self.depth = outer + 1;
                visit::visit_macro(self, node);
                self.depth = outer;
            }

            fn visit_ident(&mut self, node: &'ast Ident) {
                self.entered.push(Entered { node: Node::Ident(node), depth: self.depth });
            }

            $(
                fn $visit(&mut self, node: &'ast syn::$node) {
                    let outer = self.depth;
                    self.entered.push(Entered { node: Node::$kind(node), depth: outer });
                    self.depth = outer + 1;
                    visit::$visit(self, node);
                    self.depth = outer;
                }
            )*
        }
    };
}

crate::node_kinds!(preorder);

impl<'ast> Entered<'ast> {
    #[must_use]
    pub fn node(&self) -> Node<'ast> {
        self.node
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    pub fn preordered(file: &'ast File) -> Vec<Entered<'ast>> {
        let mut preorder = Preorder { entered: Vec::new(), depth: 0 };
        visit::visit_file(&mut preorder, file);
        preorder.entered
    }
}
