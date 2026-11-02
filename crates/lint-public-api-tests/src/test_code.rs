use syn::visit::{self, Visit};
use syn::{Attribute, ImplItemFn, ItemFn, ItemImpl, ItemMod, Macro};

use std::collections::BTreeSet;

use lint_core::Macros;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_layout::TestMarks;

#[derive(Debug)]
pub(crate) struct TestCode<'emitters> {
    emitters: &'emitters BTreeSet<Name>,
    lines: Vec<LineNumber>,
}

impl<'emitters> TestCode<'emitters> {
    const DEFINITION: &'static str = "macro_rules";

    pub(crate) fn lines_in(file: &syn::File, emitters: &'emitters BTreeSet<Name>) -> Vec<LineNumber> {
        let mut code = TestCode { emitters, lines: Vec::new() };
        code.visit_file(file);
        code.lines
    }

    fn gated(&mut self, attributes: &[Attribute], line: LineNumber) -> Walk {
        let walk = Walk::outside_tests(attributes);
        self.lines.extend((walk == Walk::Skipped).then_some(line));
        walk
    }
}

impl<'ast> Visit<'ast> for TestCode<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => visit::visit_item_mod(self, module),
            Walk::Skipped => self.lines.push(LineNumber::from(module.ident.span())),
        }
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        match self.gated(&item.attrs, LineNumber::from(item.impl_token.span)) {
            Walk::Entered => visit::visit_item_impl(self, item),
            Walk::Skipped => {}
        }
    }

    fn visit_impl_item_fn(&mut self, method: &'ast ImplItemFn) {
        match self.gated(&method.attrs, LineNumber::from(method.sig.ident.span())) {
            Walk::Entered => visit::visit_impl_item_fn(self, method),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let line = LineNumber::from(function.sig.ident.span());
        match self.gated(&function.attrs, line) {
            Walk::Entered => {
                self.lines.extend(TestMarks::mark_test(&function.attrs).then_some(line));
                visit::visit_item_fn(self, function);
            }
            Walk::Skipped => {}
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.lines.extend(
            (TestMarks::is_law(invocation)
                || (!invocation.path.is_ident(Self::DEFINITION)
                    && Macros::attributes(invocation).iter().any(TestMarks::is_test))
                || invocation.path.get_ident().is_some_and(|name| self.emitters.contains(&Name::from(name))))
            .then(|| LineNumber::from(invocation.bang_token.span)),
        );
    }
}
