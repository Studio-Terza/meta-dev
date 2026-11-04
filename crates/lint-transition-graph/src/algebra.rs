use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use law_facts::{Law, Property};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::{Mutation, Transition, Wrapping};

use crate::count::Count;
use crate::edge::Edge;
use crate::end::Node;
use crate::form::{Acting, Form};
use crate::listed::Listed;

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

impl fmt::Display for Totality {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Totality::Total => Ok(()),
            Totality::Partial => write!(formatter, "partial "),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature<'op, 'found> {
    totality: Totality,
    mutation: Mutation,
    acting: &'op Acting<'found>,
}

impl fmt::Display for Signature<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}{}", self.acting, self.totality, self.acting.subject())
    }
}

#[derive(Clone, Debug)]
pub struct Twins<'op, 'found, 'file> {
    subject: Node<'found>,
    signature: Signature<'op, 'found>,
    laws: BTreeSet<Property>,
    members: Vec<&'op Operation<'found, 'file>>,
}

struct Named<'law>(&'law Property);

impl fmt::Display for Named<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl fmt::Display for Twins<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let functions = Listed::<_, ','>::joined(self.members.iter().map(|member| member.function()));
        let laws = Listed::<_, ','>::joined(self.laws.iter().map(Named));
        write!(formatter, "{} {}: {functions} ({laws})", self.subject, self.signature)
    }
}

impl<'op, 'found, 'file> Twins<'op, 'found, 'file> {
    #[must_use]
    pub fn subject(&self) -> &Node<'found> {
        &self.subject
    }

    #[must_use]
    pub fn members(&self) -> &[&'op Operation<'found, 'file>] {
        &self.members
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
    fn of_edge(ordinal: Count, edge: &Edge<'found, 'file>) -> Option<Operation<'found, 'file>> {
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

    #[must_use]
    pub fn signature(&self) -> Signature<'_, 'found> {
        Signature { totality: self.partial, mutation: self.transition.function().mutation(), acting: &self.acting }
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

    pub(crate) fn operating(&self, law: &Law<'_>, owner: &Name) -> Option<&Operation<'found, 'file>> {
        self.operations.iter().find(|operation| law.routed(owner, operation.function()).is_some())
    }

    pub fn twins<'op, 'laws>(
        &'op self,
        laws: impl Fn(&Operation<'found, 'file>) -> &'laws [Property],
    ) -> Vec<Twins<'op, 'found, 'file>> {
        let grouped = self
            .operations
            .iter()
            .map(|operation| (operation, laws(operation).iter().copied().collect::<BTreeSet<Property>>()))
            .filter(|(_, held)| held.first().is_some())
            .fold(
                BTreeMap::new(),
                |mut groups: BTreeMap<
                    (Signature<'op, 'found>, BTreeSet<Property>),
                    Vec<&'op Operation<'found, 'file>>,
                >,
                 (operation, held)| {
                    groups.entry((operation.signature(), held)).or_default().push(operation);
                    groups
                },
            );
        grouped
            .into_iter()
            .filter(|(_, members)| members.len() > 1)
            .map(|((signature, laws), members)| Twins {
                subject: *signature.acting.subject(),
                signature,
                laws,
                members,
            })
            .collect()
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

    pub(crate) fn operating(&self, law: &Law<'_>, node: &Node<'found>) -> Option<&Operation<'found, 'file>> {
        self.of_type(node).and_then(|algebra| algebra.operating(law, AsRef::<Name>::as_ref(node)))
    }

    pub(crate) fn algebras(&self) -> impl Iterator<Item = &Algebra<'found, 'file>> {
        self.by_type.values()
    }

    pub(crate) fn operations(&self) -> impl Iterator<Item = &Operation<'found, 'file>> {
        self.by_type.values().flat_map(Algebra::operations)
    }

    pub fn types(&self) -> impl Iterator<Item = &Node<'found>> {
        self.by_type.keys()
    }
}
