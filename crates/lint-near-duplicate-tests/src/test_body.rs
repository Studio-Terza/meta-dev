mod finding;

use std::collections::BTreeMap;

use lint_core::Site;
use lint_core::Violation;
use lint_layout::Name;

use crate::body_shape::BodyShape;
use finding::Finding;
use lint_placement::PackageRoot;

#[derive(Debug)]
pub(crate) struct TestBody<'file> {
    name: Name,
    site: Site<'file>,
    shape: BodyShape,
}

impl<'file> TestBody<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, shape: BodyShape) -> TestBody<'file> {
        TestBody { name, site, shape }
    }

    pub(crate) fn filed(
        self,
        package: &'file PackageRoot,
        groups: &mut BTreeMap<&'file PackageRoot, BTreeMap<BodyShape, Vec<Repeat<'file>>>>,
    ) {
        let shapes = groups.entry(package).or_default();
        shapes.entry(self.shape).or_default().push(Repeat { name: self.name, site: self.site });
    }
}

#[derive(Debug)]
pub(crate) struct Repeat<'file> {
    name: Name,
    site: Site<'file>,
}

impl<'file> Repeat<'file> {
    pub(crate) fn repeating(&self, first: &Repeat<'_>) -> Violation<'file> {
        Violation::new(self.site, Finding::new(&self.name, &first.name))
    }
}
