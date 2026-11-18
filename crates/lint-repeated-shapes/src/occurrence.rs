use std::ops::RangeInclusive;

use body_shape::{Labels, Literals, Reading, Shape, TokenCount};
use equivalence::{Keyed, Nested};
use lint_core::Site;
use syntax_lexer::Place;

use crate::extent::Extent;
use crate::kind::Kind;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Exact {
    shape: Shape,
    labels: Labels,
}

impl From<&Exact> for TokenCount {
    fn from(exact: &Exact) -> TokenCount {
        exact.shape.size()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Occurrence<'file> {
    shape: Shape,
    labels: Labels,
    literals: Literals,
    site: Site<'file>,
    extent: Extent,
    kind: Kind,
}

impl<'file> Occurrence<'file> {
    pub(crate) fn new(reading: Reading, site: Site<'file>, extent: Extent, kind: Kind) -> Occurrence<'file> {
        Occurrence {
            shape: reading.shape(),
            labels: reading.labels().clone(),
            literals: reading.into_literals(),
            site,
            extent,
            kind,
        }
    }

    pub(crate) fn literals(&self) -> &Literals {
        &self.literals
    }

    pub(crate) fn kind(&self) -> Kind {
        self.kind
    }
}

impl Keyed for Occurrence<'_> {
    type Key = Exact;

    fn key(&self) -> Exact {
        Exact { shape: self.shape, labels: self.labels.clone() }
    }
}

impl<'file> Nested for Occurrence<'file> {
    fn lies_within<'outer>(&self, outer: &'outer Occurrence<'file>) -> Option<&'outer Occurrence<'file>> {
        let inner = RangeInclusive::<Place>::from(self.extent);
        Some(outer)
            .filter(|candidate| self.site.path() == candidate.site.path())
            .filter(|same_file| RangeInclusive::<Place>::from(same_file.extent).contains(inner.start()))
            .filter(|same_file| RangeInclusive::<Place>::from(same_file.extent).contains(inner.end()))
    }
}

impl<'file> AsRef<Site<'file>> for Occurrence<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
