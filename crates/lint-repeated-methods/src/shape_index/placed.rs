use std::collections::BTreeMap;

use lint_core::{DeclaredType, Site};

use super::method_shape::MethodShape;

#[derive(Debug)]
pub struct Placed {
    owner: DeclaredType,
    shape: MethodShape,
    site: Site,
}

impl Placed {
    pub fn new(owner: DeclaredType, shape: MethodShape, site: Site) -> Placed {
        Placed { owner, shape, site }
    }

    pub fn recorded_in(self, shapes: &mut BTreeMap<MethodShape, BTreeMap<DeclaredType, Site>>) {
        shapes.entry(self.shape).or_default().entry(self.owner).or_insert(self.site);
    }
}
