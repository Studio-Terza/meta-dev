use lint_core::Site;

use crate::extent::Extent;

#[derive(Clone, Debug)]
pub(crate) struct Occurrence<'file> {
    site: Site<'file>,
    extent: Extent,
}

impl<'file> Occurrence<'file> {
    pub(crate) fn new(site: Site<'file>, extent: Extent) -> Occurrence<'file> {
        Occurrence { site, extent }
    }

    pub(crate) fn lies_within<'outer>(&self, outer: &'outer Occurrence<'file>) -> Option<&'outer Occurrence<'file>> {
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
