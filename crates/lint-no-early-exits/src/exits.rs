use proc_macro2::Span;
use syn::visit::{self, Visit};
use syn::{ExprBreak, ExprContinue, ExprReturn, Macro};

use lint_core::FileLint;
use lint_core::{Macros, Site, SourceFile, Violation};
use lint_layout::LineNumber;

use super::Finding;

pub struct Exits<'file> {
    file: &'file SourceFile,
    found: Vec<Violation<'file>>,
}

impl Exits<'_> {
    fn record(&mut self, span: Span, finding: Finding) {
        self.found.push(Violation::new(Site::new(self.file.path(), LineNumber::from(span)), finding));
    }
}

impl FileLint for Exits<'_> {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut scan = Exits { file, found: Vec::new() };
        scan.visit_file(file.syntax());
        scan.found.extend(file.test_modules().iter().flat_map(Exits::violations));
        scan.found
    }
}

impl<'ast> Visit<'ast> for Exits<'_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_expr_return(&mut self, expr: &'ast ExprReturn) {
        self.record(expr.return_token.span, Finding::Return);
        visit::visit_expr_return(self, expr);
    }

    fn visit_expr_break(&mut self, expr: &'ast ExprBreak) {
        self.record(expr.break_token.span, Finding::Break);
        visit::visit_expr_break(self, expr);
    }

    fn visit_expr_continue(&mut self, expr: &'ast ExprContinue) {
        self.record(expr.continue_token.span, Finding::Continue);
        visit::visit_expr_continue(self, expr);
    }
}
