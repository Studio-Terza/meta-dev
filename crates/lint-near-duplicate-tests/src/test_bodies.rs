use syn::ItemFn;
use syn::visit::{self, Visit};

use lint_core::{Site, SourceFile};
use lint_layout::{LineNumber, Name, TestMarks};

use crate::body_shape::BodyShape;
use crate::test_body::TestBody;

#[derive(Debug)]
pub(crate) struct TestBodies<'file> {
    file: &'file SourceFile,
    bodies: Vec<TestBody>,
}

impl TestBodies<'_> {
    pub(crate) fn found(file: &SourceFile) -> Vec<TestBody> {
        let mut scan = TestBodies { file, bodies: Vec::new() };
        scan.visit_file(file.syntax());
        scan.bodies
    }
}

impl TestBodies<'_> {
    const PLAIN_TEST: &'static str = "test";
    const PANICKING: &'static str = "should_panic";

    fn is_row_shaped(function: &ItemFn) -> bool {
        TestMarks::mark_test(&function.attrs)
            && function.sig.asyncness.is_none()
            && function.attrs.iter().all(|attribute| {
                !attribute.path().is_ident(Self::PANICKING)
                    && (!TestMarks::is_test(attribute) || attribute.path().is_ident(Self::PLAIN_TEST))
            })
    }
}

impl<'ast> Visit<'ast> for TestBodies<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let name = &function.sig.ident;
        self.bodies.extend(Self::is_row_shaped(function).then(|| {
            let site = Site::new(self.file.path(), LineNumber::from(name.span()));
            TestBody::new(Name::from(name), site, BodyShape::from(function))
        }));
        visit::visit_item_fn(self, function);
    }
}
