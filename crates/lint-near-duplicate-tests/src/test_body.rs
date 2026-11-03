mod finding;

use lint_core::Site;
use lint_core::Violation;
use lint_layout::Name;

use crate::body_shape::BodyShape;
use finding::Finding;

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

    pub(crate) fn shape(&self) -> &BodyShape {
        &self.shape
    }

    pub(crate) fn repeating(&self, first: &TestBody<'_>) -> Violation<'file> {
        Violation::new(self.site, Finding::new(&self.name, &first.name))
    }
}
