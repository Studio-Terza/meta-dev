use std::ptr;

use lint_core::Site;
use transition_facts::Transition;

use super::label::Label;
use crate::edge::Edge;
use crate::end::Hop;

#[derive(Debug)]
pub(crate) struct Hyperedge<'found, 'file> {
    hop: Hop,
    label: Label,
    transition: &'found Transition<'file>,
}

impl<'found, 'file> Hyperedge<'found, 'file> {
    pub(super) fn of_edge(edge: &Edge<'found, 'file>) -> Option<Hyperedge<'found, 'file>> {
        edge.ends().hop().and_then(Hop::onward).map(|hop| Hyperedge {
            hop,
            label: Label::read(edge.role(), edge.transition()),
            transition: edge.transition(),
        })
    }

    pub(crate) fn written(&self) -> Option<&Hyperedge<'found, 'file>> {
        self.label.written().map(|_| self)
    }

    pub(crate) fn other<'edge>(
        &'edge self,
        than: &Hyperedge<'found, 'file>,
    ) -> Option<&'edge Hyperedge<'found, 'file>> {
        (!ptr::eq(self, than)).then_some(self)
    }

    pub(crate) fn transition(&self) -> &'found Transition<'file> {
        self.transition
    }
}

impl AsRef<Hop> for Hyperedge<'_, '_> {
    fn as_ref(&self) -> &Hop {
        &self.hop
    }
}

impl AsRef<Label> for Hyperedge<'_, '_> {
    fn as_ref(&self) -> &Label {
        &self.label
    }
}

impl<'file> AsRef<Site<'file>> for Hyperedge<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.transition.as_ref()
    }
}
