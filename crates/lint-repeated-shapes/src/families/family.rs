mod finding;

use lint_core::{Site, Violation};

use crate::kind::Kind;
use crate::occurrence::Occurrence;
use body_shape::TokenCount;
use finding::Finding;

#[derive(Clone, Debug)]
pub(crate) struct Family<'file> {
    size: TokenCount,
    copies: Vec<Occurrence<'file>>,
}

impl<'file> Family<'file> {
    pub(crate) fn new(size: TokenCount, copies: Vec<Occurrence<'file>>) -> Family<'file> {
        Family { size, copies }
    }

    pub(crate) fn size(&self) -> TokenCount {
        self.size
    }

    pub(crate) fn copies(&self) -> &[Occurrence<'file>] {
        &self.copies
    }

    pub(crate) fn violation(&self) -> Violation<'file> {
        let mut sites: Vec<&Site<'file>> = self.copies.iter().map(Occurrence::as_ref).collect();
        sites.sort();
        let kind = self.copies.iter().map(Occurrence::kind).reduce(Kind::joined).unwrap_or(Kind::Mixed);
        sites.first().map_or_else(
            || unreachable!("a family has copies"),
            |site| Violation::new(**site, Finding::new(self.size, kind, sites.get(1..).unwrap_or_default())),
        )
    }
}
