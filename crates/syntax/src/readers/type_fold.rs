use std::cell::RefCell;
use std::ptr;

use syn::visit::{self, Visit};
use syn::{Generics, Type, TypeParamBound};

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
struct Folded<'syn, Out> {
    node: &'syn Type,
    out: RefCell<Option<Out>>,
}

#[derive(Debug)]
pub struct TypeFold<'syn, Out> {
    done: Vec<Folded<'syn, Out>>,
}

impl<'syn, Out> TypeFold<'syn, Out> {
    #[must_use]
    pub fn over(tree: TypeTree<'syn>, step: impl Fn(&TypeFold<'syn, Out>, &'syn Type) -> Out) -> TypeFold<'syn, Out> {
        tree.postorder.into_iter().fold(TypeFold { done: Vec::new() }, |mut fold, node| {
            let out = RefCell::new(Some(step(&fold, node)));
            fold.done.push(Folded { node, out });
            fold
        })
    }

    #[must_use]
    pub fn take(&self, ty: &Type) -> Option<Out> {
        self.done.iter().find(|folded| ptr::eq(folded.node, ty)).and_then(|folded| folded.out.borrow_mut().take())
    }
}
