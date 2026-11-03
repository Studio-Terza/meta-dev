use std::collections::BTreeMap;

use lint_core::{DeclaredType, Site};

use super::method_shape::MethodShape;

#[derive(Debug)]
pub struct Placed<'file> {
    owner: DeclaredType,
    shape: MethodShape,
    site: Site<'file>,
}

impl<'file> Placed<'file> {
    pub fn new(owner: DeclaredType, shape: MethodShape, site: Site<'file>) -> Placed<'file> {
        Placed { owner, shape, site }
    }

    pub fn recorded_in(self, shapes: &mut BTreeMap<MethodShape, BTreeMap<DeclaredType, Site<'file>>>) {
        shapes.entry(self.shape).or_default().entry(self.owner).or_insert(self.site);
    }
}
