use syn::visit::{self, Visit};
use syn::{Attribute, ItemFn};

use lint_core::Site;
use lint_core::SourceFile;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::TestMarks;

use crate::body_shape::BodyShape;
use crate::test_body::TestBody;

#[derive(Debug)]
pub(crate) struct TestBodies<'file> {
    file: &'file SourceFile,
    bodies: Vec<TestBody<'file>>,
}

impl<'file> TestBodies<'file> {
    pub(crate) fn found(file: &'file SourceFile) -> Vec<TestBody<'file>> {
        let mut scan = TestBodies { file, bodies: Vec::new() };
        scan.visit_file(file.syntax());
        scan.bodies
    }
}

impl TestBodies<'_> {
    const PLAIN_TEST: &'static str = "test";
    const PANICKING: &'static str = "should_panic";

    fn row_shaped(function: &ItemFn) -> Option<&ItemFn> {
        TestMarks::mark_test(&function.attrs)
            .filter(|_| function.sig.asyncness.is_none())
            .filter(|_| function.attrs.iter().all(|attribute| Self::row_attribute(attribute).is_some()))
            .map(|_| function)
    }

    fn row_attribute(attribute: &Attribute) -> Option<&Attribute> {
        Some(attribute)
            .filter(|attribute| attribute.path().get_ident().is_none_or(|ident| ident != Self::PANICKING))
            .filter(|attribute| TestMarks::test(attribute).is_none_or(|_| attribute.path().is_ident(Self::PLAIN_TEST)))
    }
}

impl<'ast> Visit<'ast> for TestBodies<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let name = &function.sig.ident;
        self.bodies.extend(Self::row_shaped(function).map(|_| {
            let site = Site::new(self.file.path(), LineNumber::from(name.span()));
            TestBody::new(Name::from(name), site, BodyShape::from(function))
        }));
        visit::visit_item_fn(self, function);
    }
}
