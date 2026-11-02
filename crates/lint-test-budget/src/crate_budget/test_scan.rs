use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro, PathSegment};

use lint_core::Macros;
use lint_core::Walk;
use lint_layout::TestMarks;
use newtype::Counter;

use super::CrateBudget;
use super::count::Count;

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
        Count::new(span.end().line - span.start().line + 1)
    }

    fn property_block(invocation: &Macro) -> Option<&PathSegment> {
        invocation.path.segments.last().filter(|segment| segment.ident == Self::PROPERTY_BLOCK)
    }

    fn tests_in(invocation: &Macro) -> Count {
        match (TestMarks::law(invocation), Self::property_block(invocation)) {
            (Some(_), _) => Count::new(1),
            (None, Some(_)) => Count::new(Macros::attributes(invocation).iter().filter_map(TestMarks::test).count()),
            (None, None) => Count::default(),
        }
    }
}

impl<'ast> Visit<'ast> for TestScan {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.0.tests = self.0.tests.plus(Count::new(TestMarks::mark_test(&function.attrs).into_iter().count()));
        visit::visit_item_fn(self, function);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let before = self.0.test_lines;
        visit::visit_item_mod(self, module);
        match Walk::outside_tests(&module.attrs) {
            Walk::Skipped => self.0.test_lines = before.plus(Self::line_count(module.span())),
            Walk::Entered => {}
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.0.tests = self.0.tests.plus(Self::tests_in(invocation));
    }
}
