use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

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
}

impl<'ast> Visit<'ast> for TestCode<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => visit::visit_item_mod(self, module),
            Walk::Skipped => self.lines.push(LineNumber::from(module.ident.span())),
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.lines.extend(TestMarks::mark_test(&function.attrs).then(|| LineNumber::from(function.sig.ident.span())));
        visit::visit_item_fn(self, function);
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
