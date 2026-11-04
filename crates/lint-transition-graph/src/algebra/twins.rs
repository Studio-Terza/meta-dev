use std::collections::BTreeSet;
use std::fmt;

use equivalence::{Equivalence, Erased, Group, Groups, Keyed};
use law_facts::Property;
use lint_core::Site;
use lint_layout::Name;

use super::{Operation, Signature};
use crate::end::Node;
use crate::listed::Listed;

#[derive(Clone, Copy, Debug)]
pub struct Lawed<'op, 'found, 'file> {
    operation: &'op Operation<'found, 'file>,
    laws: &'op [Property],
    equivalence: Equivalence,
}

impl<'op, 'found, 'file> Lawed<'op, 'found, 'file> {
    pub(super) fn new(
        operation: &'op Operation<'found, 'file>,
        laws: &'op [Property],
        equivalence: Equivalence,
    ) -> Lawed<'op, 'found, 'file> {
        Lawed { operation, laws, equivalence }
    }

    fn declared(self) -> Option<Lawed<'op, 'found, 'file>> {
        self.laws.first().map(|_| self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Twinning<'op, 'found> {
    signature: Signature<'op, 'found>,
    laws: BTreeSet<Property>,
    name: Option<&'op Name>,
}

impl<'op, 'found> Keyed for Lawed<'op, 'found, '_> {
    type Key = Twinning<'op, 'found>;

    fn key(&self) -> Twinning<'op, 'found> {
        Twinning {
            signature: self.operation.signature(),
            laws: self.equivalence.kept(self.laws).iter().copied().collect(),
            name: self.equivalence.erases(Erased::Names).map_or(Some(self.operation.function()), |_| None),
        }
    }
}

impl<'file> AsRef<Site<'file>> for Lawed<'_, '_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.operation.as_ref()
    }
}

#[derive(Debug)]
pub struct Twins<'op, 'found, 'file>(Group<Lawed<'op, 'found, 'file>>);

impl<'op, 'found, 'file> Twins<'op, 'found, 'file> {
    pub(super) fn grouped(
        operations: impl Iterator<Item = Lawed<'op, 'found, 'file>>,
    ) -> Vec<Twins<'op, 'found, 'file>> {
        Groups::grouped(operations.filter_map(Lawed::declared)).listed().into_iter().map(Twins).collect()
    }

    #[must_use]
    pub fn subject(&self) -> &Node<'found> {
        self.0.key().signature.acting.subject()
    }

    #[must_use]
    pub fn group(&self) -> &Group<Lawed<'op, 'found, 'file>> {
        &self.0
    }
}

struct Named<'law>(&'law Property);

impl fmt::Display for Named<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}

impl fmt::Display for Twins<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let functions = Listed::<_, ','>::joined(self.0.members().iter().map(|member| member.operation.function()));
        let laws = Listed::<_, ','>::joined(self.0.key().laws.iter().map(Named));
        write!(formatter, "{} {}: {functions} ({laws})", self.subject(), self.0.key().signature)
    }
}
