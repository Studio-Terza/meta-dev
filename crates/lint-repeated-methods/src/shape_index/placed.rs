use std::collections::BTreeMap;

use lint_core::Site;
use lint_layout::Name;

use super::method_shape::MethodShape;

#[derive(Debug)]
pub struct Placed {
    owner: Name,
    shape: MethodShape,
    site: Site,
}

impl Placed {
    pub fn new(owner: Name, shape: MethodShape, site: Site) -> Placed {
        Placed { owner, shape, site }
    }

    pub fn recorded_in(self, shapes: &mut BTreeMap<MethodShape, BTreeMap<Name, Site>>) {
        shapes.entry(self.shape).or_default().entry(self.owner).or_insert(self.site);
    }
}
