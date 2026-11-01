use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use lint_core::{Macros, TestMarks, Walk};

use crate::count::Count;
use crate::crate_budget::CrateBudget;

#[derive(Debug, Default)]
pub(super) struct TestScan(CrateBudget);

impl TestScan {
    const PROPERTY_BLOCK: &str = "proptest";

    pub(super) fn measured(file: &syn::File) -> CrateBudget {
        let mut scan = TestScan::default();
        scan.visit_file(file);
        scan.0
    }

    fn line_count(span: Span) -> Count {
        Count::from(span.end().line - span.start().line + 1)
    }

    fn is_property_block(invocation: &Macro) -> bool {
        invocation.path.segments.last().is_some_and(|segment| segment.ident == Self::PROPERTY_BLOCK)
    }

    fn tests_in(invocation: &Macro) -> Count {
        if TestMarks::is_law(invocation) {
            Count::from(1)
        } else if Self::is_property_block(invocation) {
            Count::from(Macros::attributes(invocation).iter().filter(|attribute| TestMarks::is_test(attribute)).count())
        } else {
            Count::default()
        }
    }
}

impl<'ast> Visit<'ast> for TestScan {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.0.tests = self.0.tests + Count::from(usize::from(TestMarks::mark_test(&function.attrs)));
        visit::visit_item_fn(self, function);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let before = self.0.test_lines;
        visit::visit_item_mod(self, module);
        match Walk::outside_tests(&module.attrs) {
            Walk::Skipped => self.0.test_lines = before + Self::line_count(module.span()),
            Walk::Entered => {}
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.0.tests = self.0.tests + Self::tests_in(invocation);
    }
}
