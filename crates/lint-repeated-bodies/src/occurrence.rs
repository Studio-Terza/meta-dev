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

    pub(crate) fn lies_within(&self, outer: &Occurrence) -> bool {
        self.site.path() == outer.site.path() && outer.extent.holds(&self.extent)
    }
}

impl AsRef<Site> for Occurrence {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
