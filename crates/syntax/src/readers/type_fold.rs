use std::cell::RefCell;
use std::ptr;

use syn::visit::{self, Visit};
use syn::{Expr, Generics, Type, TypeParamBound};

#[derive(Debug, Default)]
pub struct TypeTree<'syn> {
    postorder: Vec<&'syn Type>,
}

impl<'syn> TypeTree<'syn> {
    #[must_use]
    pub fn bounds(bounds: &[&'syn TypeParamBound]) -> TypeTree<'syn> {
        bounds.iter().fold(TypeTree::default(), |mut tree, bound| {
            tree.visit_type_param_bound(bound);
            tree
        })
    }
}

#[derive(Debug, Default)]
struct Outermost<'syn>(Vec<&'syn Type>);

impl<'syn> Visit<'syn> for Outermost<'syn> {
    fn visit_type(&mut self, ty: &'syn Type) {
        self.0.push(ty);
    }
}

impl<'syn> TypeTree<'syn> {
    #[must_use]
    pub fn outermost(expr: &'syn Expr) -> Vec<&'syn Type> {
        let mut found = Outermost::default();
        found.visit_expr(expr);
        found.0
    }
}

impl<'syn> From<&'syn Type> for TypeTree<'syn> {
    fn from(ty: &'syn Type) -> TypeTree<'syn> {
        let mut tree = TypeTree::default();
        tree.visit_type(ty);
        tree
    }
}

impl<'syn> From<&'syn Generics> for TypeTree<'syn> {
    fn from(generics: &'syn Generics) -> TypeTree<'syn> {
        let mut tree = TypeTree::default();
        tree.visit_generics(generics);
        tree
    }
}

impl<'syn> Visit<'syn> for TypeTree<'syn> {
    fn visit_type(&mut self, ty: &'syn Type) {
        visit::visit_type(self, ty);
        self.postorder.push(ty);
    }
}

#[derive(Debug)]
struct Folded<'syn, Node, Out> {
    node: &'syn Node,
    out: RefCell<Option<Out>>,
}

#[derive(Debug)]
pub struct Fold<'syn, Node, Out> {
    done: Vec<Folded<'syn, Node, Out>>,
}

impl<'syn, Node, Out> Fold<'syn, Node, Out> {
    #[must_use]
    pub fn over(
        tree: impl IntoIterator<Item = &'syn Node>,
        step: impl Fn(&Fold<'syn, Node, Out>, &'syn Node) -> Out,
    ) -> Fold<'syn, Node, Out> {
        tree.into_iter().fold(Fold { done: Vec::new() }, |mut fold, node| {
            let out = RefCell::new(Some(step(&fold, node)));
            fold.done.push(Folded { node, out });
            fold
        })
    }

    #[must_use]
    pub fn take(&self, node: &Node) -> Option<Out> {
        self.done.iter().find(|folded| ptr::eq(folded.node, node)).and_then(|folded| folded.out.borrow_mut().take())
    }
}

impl<'syn> IntoIterator for TypeTree<'syn> {
    type Item = &'syn Type;
    type IntoIter = std::vec::IntoIter<&'syn Type>;

    fn into_iter(self) -> Self::IntoIter {
        self.postorder.into_iter()
    }
}

#[derive(Debug, Default)]
pub struct ExprTree<'syn> {
    postorder: Vec<&'syn Expr>,
}

impl<'syn> From<&'syn Expr> for ExprTree<'syn> {
    fn from(expr: &'syn Expr) -> ExprTree<'syn> {
        let mut tree = ExprTree::default();
        tree.visit_expr(expr);
        tree
    }
}

impl<'syn> Visit<'syn> for ExprTree<'syn> {
    fn visit_expr(&mut self, expr: &'syn Expr) {
        visit::visit_expr(self, expr);
        self.postorder.push(expr);
    }
}

impl<'syn> IntoIterator for ExprTree<'syn> {
    type Item = &'syn Expr;
    type IntoIter = std::vec::IntoIter<&'syn Expr>;

    fn into_iter(self) -> Self::IntoIter {
        self.postorder.into_iter()
    }
}
