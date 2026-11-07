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

    fn entered(&mut self, function: &ItemFn) {
        let line = LineNumber::from(function.sig.ident.span());
        self.lines.extend(TestMarks::mark_test(&function.attrs).map(|_| line));
        visit::visit_item_fn(self, function);
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
        match self.gated(&function.attrs, LineNumber::from(function.sig.ident.span())) {
            Walk::Entered => self.entered(function),
            Walk::Skipped => {}
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let line = LineNumber::from(invocation.bang_token.span);
        let law = TestMarks::law(invocation).map(|_| line);
        let called = Some(invocation)
            .filter(|invocation| invocation.path.get_ident().is_none_or(|name| name != Self::DEFINITION));
        let marked = called.and_then(|call| Macros::attributes(call).iter().find_map(TestMarks::test).map(|_| line));
        let emitted =
            invocation.path.get_ident().filter(|name| self.emitters.contains(&Name::from(*name))).map(|_| line);
        self.lines.extend(law.or(marked).or(emitted));
    }
}
