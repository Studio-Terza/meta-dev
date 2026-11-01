mod finding;

use lint_core::{Name, Site, Violation};

use crate::body_shape::BodyShape;
use finding::Finding;

#[derive(Debug)]
pub(crate) struct TestBody {
    name: Name,
    site: Site,
    shape: BodyShape,
}

impl TestBody {
    pub(crate) fn new(name: Name, site: Site, shape: BodyShape) -> TestBody {
        TestBody { name, site, shape }
    }

    pub(crate) fn shape(&self) -> &BodyShape {
        &self.shape
    }

    pub(crate) fn repeating(&self, first: &TestBody) -> Violation {
        Violation::new(self.site.clone(), Finding::new(self.name.clone(), first.name.clone()))
    }
}
