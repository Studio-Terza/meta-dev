use std::slice;

use syn::Attribute;
use syn::visit::{self, Visit};

use lint_core::Walk;
use lint_layout::LineNumber;

#[derive(Debug, Default)]
pub(crate) struct TestGates(Vec<LineNumber>);

impl TestGates {
    pub(crate) fn lines_in(file: &syn::File) -> Vec<LineNumber> {
        let mut gates = TestGates::default();
        gates.visit_file(file);
        gates.0
    }
}

impl<'ast> Visit<'ast> for TestGates {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        let gated = Some(attribute)
            .filter(|candidate| Walk::outside_tests(slice::from_ref(*candidate)) == Walk::Skipped)
            .map(|skipped| LineNumber::from(skipped.pound_token.span));
        self.0.extend(gated);
        visit::visit_attribute(self, attribute);
    }
}
