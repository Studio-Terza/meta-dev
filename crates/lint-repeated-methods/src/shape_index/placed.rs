use std::collections::BTreeMap;

use lint_core::{DeclaredType, Site};

use super::method_shape::MethodShape;

#[derive(Debug)]
pub struct Placed<'file> {
    owner: DeclaredType<'file>,
    shapes: BTreeMap<MethodShape, Site<'file>>,
}

impl<'file> Placed<'file> {
    pub const fn new(owner: DeclaredType<'file>, shapes: BTreeMap<MethodShape, Site<'file>>) -> Self {
        Placed { owner, shapes }
    }

    pub fn recorded_in(self, owners: &mut BTreeMap<DeclaredType<'file>, BTreeMap<MethodShape, Site<'file>>>) {
        self.shapes.into_iter().fold(owners.entry(self.owner).or_default(), |recorded, (shape, site)| {
            recorded.entry(shape).or_insert(site);
            recorded
        });
    }
}
