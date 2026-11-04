use std::collections::BTreeMap;

use transition_facts::{Transition, Wrapping};

use crate::edge::Edge;
use crate::end::Node;
use crate::form::{Acting, Form};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Totality {
    Total,
    Partial,
}

impl Totality {
    pub(crate) fn of_wrapping(wrapping: Wrapping) -> Option<Totality> {
        match wrapping {
            Wrapping::Optional | Wrapping::Fallible => Some(Totality::Partial),
            Wrapping::Plain => Some(Totality::Total),
            Wrapping::Many => None,
        }
    }

    pub(crate) fn shaping(self, acting: Acting<'_>) -> Form<'_> {
        match self {
            Totality::Partial => Form::Partial(acting),
            Totality::Total => Form::Operation(acting),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operation<'found, 'file> {
    transition: Transition<'found, 'file>,
    acting: Acting<'found>,
    partial: Totality,
}

impl<'found, 'file> Operation<'found, 'file> {
    fn of_edge(edge: &Edge<'found, 'file>) -> Option<Operation<'found, 'file>> {
        let transition = *edge.transition();
        let partial = Totality::of_wrapping(transition.function().wrapping())?;
        edge.form().algebraic().and(Acting::of_edge(edge)).map(|acting| Operation { transition, acting, partial })
    }

    #[must_use]
    pub fn transition(&self) -> &Transition<'found, 'file> {
        &self.transition
    }

    #[must_use]
    pub fn partial(&self) -> Totality {
        self.partial
    }
}

#[derive(Clone, Debug, Default)]
pub struct Algebra<'found, 'file> {
    operations: Vec<Operation<'found, 'file>>,
}

impl<'found, 'file> Algebra<'found, 'file> {
    #[must_use]
    pub fn operations(&self) -> &[Operation<'found, 'file>] {
        &self.operations
    }
}

#[derive(Clone, Debug, Default)]
pub struct Algebras<'found, 'file> {
    by_type: BTreeMap<Node<'found>, Algebra<'found, 'file>>,
}

impl<'found, 'file> Algebras<'found, 'file> {
    pub(crate) fn of_edges<'edge>(edges: impl Iterator<Item = &'edge Edge<'found, 'file>>) -> Algebras<'found, 'file>
    where
        'found: 'edge,
        'file: 'found,
    {
        let by_type = edges.filter_map(Operation::of_edge).fold(
            BTreeMap::new(),
            |mut types: BTreeMap<Node<'found>, Algebra<'found, 'file>>, operation| {
                types.entry(*operation.acting.subject()).or_default().operations.push(operation);
                types
            },
        );
        Algebras { by_type }
    }

    #[must_use]
    pub fn of_type(&self, node: &Node<'found>) -> Option<&Algebra<'found, 'file>> {
        self.by_type.get(node)
    }

    pub fn types(&self) -> impl Iterator<Item = &Node<'found>> {
        self.by_type.keys()
    }
}
