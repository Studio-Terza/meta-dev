use std::collections::BTreeMap;

use law_facts::Law;
use law_facts::law_path::{LawPath, Side};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Transition, Wrapping};

use crate::count::Count;
use crate::edge::Edge;
use crate::end::Node;
use crate::form::{Acting, Form};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Totality {
    Total,
    Partial,
}

impl Totality {
    pub(crate) fn of_wrapping(wrapping: Wrapping) -> Option<Self> {
        match wrapping {
            Wrapping::Optional | Wrapping::Fallible => Some(Self::Partial),
            Wrapping::Plain => Some(Self::Total),
            Wrapping::Many => None,
        }
    }

    pub(crate) fn shaping(self, acting: Acting<'_>) -> Form<'_> {
        match self {
            Self::Partial => Form::Partial(acting),
            Self::Total => Form::Operation(acting),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operation<'found, 'file> {
    transition: Transition<'found, 'file>,
    acting: Acting<'found>,
    partial: Totality,
    ordinal: Count,
}

impl<'found, 'file> Operation<'found, 'file> {
    fn of_edge(ordinal: Count, edge: &Edge<'found, 'file>) -> Option<Self> {
        let transition = *edge.transition();
        let partial = Totality::of_wrapping(transition.function().wrapping())?;
        edge.form().algebraic().and(Acting::of_edge(edge)).map(|acting| Operation {
            transition,
            acting,
            partial,
            ordinal,
        })
    }

    #[must_use]
    pub fn transition(&self) -> &Transition<'found, 'file> {
        &self.transition
    }

    #[must_use]
    pub fn partial(&self) -> Totality {
        self.partial
    }

    pub(crate) fn ordinal(&self) -> Count {
        self.ordinal
    }

    pub(crate) fn acting(&self) -> &Acting<'found> {
        &self.acting
    }

    pub(crate) fn function(&self) -> &Name {
        AsRef::<Name>::as_ref(self.transition.function())
    }

    pub(crate) fn routed_by<'law>(&self, law: &'law Law<'_>, side: Side) -> Option<&'law LawPath> {
        law.routed_on(side, AsRef::<Name>::as_ref(self.acting.subject()), self.function())
    }
}

impl AsRef<CrateName> for Operation<'_, '_> {
    fn as_ref(&self) -> &CrateName {
        self.transition.as_ref()
    }
}

impl<'file> AsRef<Site<'file>> for Operation<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.transition.as_ref()
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

    pub(crate) fn operating(&self, law: &Law<'_>, side: Side) -> Option<&Operation<'found, 'file>> {
        self.operations.iter().find(|operation| operation.routed_by(law, side).is_some())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Algebras<'found, 'file> {
    by_type: BTreeMap<Node<'found>, Algebra<'found, 'file>>,
}

impl<'found, 'file> Algebras<'found, 'file> {
    pub(crate) fn of_edges<'edge>(edges: impl Iterator<Item = &'edge Edge<'found, 'file>>) -> Self
    where
        'found: 'edge,
        'file: 'found,
    {
        let by_type = edges
            .enumerate()
            .filter_map(|(ordinal, edge)| Operation::of_edge(Count::new(ordinal), edge))
            .fold(BTreeMap::new(), |mut types: BTreeMap<Node<'found>, Algebra<'found, 'file>>, operation| {
                types.entry(*operation.acting.subject()).or_default().operations.push(operation);
                types
            });
        Algebras { by_type }
    }

    #[must_use]
    pub fn of_type(&self, node: &Node<'found>) -> Option<&Algebra<'found, 'file>> {
        self.by_type.get(node)
    }

    pub(crate) fn operating(
        &self,
        law: &Law<'_>,
        side: Side,
        node: &Node<'found>,
    ) -> Option<&Operation<'found, 'file>> {
        self.of_type(node).and_then(|algebra| algebra.operating(law, side))
    }

    pub(crate) fn operations(&self) -> impl Iterator<Item = &Operation<'found, 'file>> {
        self.by_type.values().flat_map(Algebra::operations)
    }

    pub fn types(&self) -> impl Iterator<Item = &Node<'found>> {
        self.by_type.keys()
    }
}
