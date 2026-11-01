use syn::visit::{self, Visit};
use syn::{ItemFn, ItemMod, Macro};

use lint_core::{LineNumber, TestMarks, Walk};

#[derive(Debug, Default)]
pub(crate) struct TestCode {
    lines: Vec<LineNumber>,
}

impl TestCode {
    pub(crate) fn lines_in(file: &syn::File) -> Vec<LineNumber> {
        let mut code = TestCode::default();
        code.visit_file(file);
        code.lines
    }
}

impl<'ast> Visit<'ast> for TestCode {
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
        self.lines.extend(TestMarks::is_law(invocation).then(|| LineNumber::from(invocation.bang_token.span)));
    }
}
