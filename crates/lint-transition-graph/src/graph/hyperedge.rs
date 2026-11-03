use std::ptr;

use lint_core::Site;
use transition_facts::Transition;

use super::label::Label;
use crate::edge::Edge;
use crate::end::Hop;

#[derive(Debug)]
pub(crate) struct Hyperedge<'found> {
    hop: Hop,
    label: Label,
    transition: &'found Transition,
}

impl<'found> Hyperedge<'found> {
    pub(super) fn of_edge(edge: &Edge<'found>) -> Option<Hyperedge<'found>> {
        edge.ends().hop().and_then(Hop::onward).map(|hop| Hyperedge {
            hop,
            label: Label::read(edge.role(), edge.transition()),
            transition: edge.transition(),
        })
    }

    pub(crate) fn written(&self) -> Option<&Hyperedge<'found>> {
        self.label.written().map(|_| self)
    }

    pub(crate) fn other<'edge>(&'edge self, than: &Hyperedge<'found>) -> Option<&'edge Hyperedge<'found>> {
        (!ptr::eq(self, than)).then_some(self)
    }

    pub(crate) fn transition(&self) -> &'found Transition {
        self.transition
    }
}

impl AsRef<Hop> for Hyperedge<'_> {
    fn as_ref(&self) -> &Hop {
        &self.hop
    }
}

impl AsRef<Label> for Hyperedge<'_> {
    fn as_ref(&self) -> &Label {
        &self.label
    }
}

impl AsRef<Site> for Hyperedge<'_> {
    fn as_ref(&self) -> &Site {
        self.transition.as_ref()
    }
}
