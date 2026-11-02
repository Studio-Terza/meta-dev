use lint_core::Site;

use crate::extent::Extent;

#[derive(Clone, Debug)]
pub(crate) struct Occurrence {
    site: Site,
    extent: Extent,
}

impl Occurrence {
    pub(crate) fn new(site: Site, extent: Extent) -> Occurrence {
        Occurrence { site, extent }
    }

    pub(crate) fn lies_within<'outer>(&self, outer: &'outer Occurrence) -> Option<&'outer Occurrence> {
        Some(outer)
            .filter(|outer| self.site.path() == outer.site.path())
            .and_then(|outer| outer.extent.holds(&self.extent).map(|_| outer))
    }
}

impl AsRef<Site> for Occurrence {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
