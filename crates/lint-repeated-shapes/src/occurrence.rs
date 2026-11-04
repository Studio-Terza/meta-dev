use body_shape::{Literals, Reading, Shape};
use equivalence::{Keyed, Nested};
use lint_core::Site;

use crate::extent::Extent;
use crate::kind::Kind;

#[derive(Clone, Debug)]
pub(crate) struct Occurrence<'file> {
    shape: Shape,
    literals: Literals,
    site: Site<'file>,
    extent: Extent,
    kind: Kind,
}

impl<'file> Occurrence<'file> {
    pub(crate) fn new(reading: Reading, site: Site<'file>, extent: Extent, kind: Kind) -> Occurrence<'file> {
        Occurrence { shape: reading.shape(), literals: reading.into_literals(), site, extent, kind }
    }

    pub(crate) fn literals(&self) -> &Literals {
        &self.literals
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }
}

impl Keyed for Occurrence<'_> {
    type Key = Shape;

    fn key(&self) -> Shape {
        self.shape
    }
}

impl<'file> Nested for Occurrence<'file> {
    fn lies_within<'outer>(&self, outer: &'outer Occurrence<'file>) -> Option<&'outer Occurrence<'file>> {
        Some(outer)
            .filter(|outer| self.site.path() == outer.site.path())
            .and_then(|outer| outer.extent.holds(&self.extent).map(|_| outer))
    }
}

impl<'file> AsRef<Site<'file>> for Occurrence<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
