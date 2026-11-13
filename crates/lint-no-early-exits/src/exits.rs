mod finding;

use proc_macro2::Span;
use syn::{ExprBreak, ExprContinue, ExprReturn, Macro};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Finds, Kind, Sweep, Watch};

use finding::Finding;

#[derive(Debug)]
pub struct Exits<'file> {
    file: &'file SourceFile,
    found: Vec<Violation<'file>>,
}

impl Exits<'_> {
    fn record(&mut self, span: Span, finding: Finding) -> Descend {
        self.found.push(Violation::new(Site::new(self.file.head().path(), LineNumber::from(span)), finding));
        Descend::Into
    }
}

impl<'file> From<&'file SourceFile> for Exits<'file> {
    fn from(file: &'file SourceFile) -> Exits<'file> {
        Exits { file, found: Vec::new() }
    }
}

impl<'ast> Watch<'ast> for Exits<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ExprReturn, Kind::ExprBreak, Kind::ExprContinue]
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_expr_return(&mut self, expr: &'ast ExprReturn) -> Descend {
        self.record(expr.return_token.span, Finding::Return)
    }

    fn on_expr_break(&mut self, expr: &'ast ExprBreak) -> Descend {
        self.record(expr.break_token.span, Finding::Break)
    }

    fn on_expr_continue(&mut self, expr: &'ast ExprContinue) -> Descend {
        self.record(expr.continue_token.span, Finding::Continue)
    }
}

impl<'file> Finds<'file> for Exits<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let modules = self.file.test_modules().iter();
        let tested = modules.flat_map(|module| Sweep::found(module, Box::new(Exits::from(module))));
        self.found.into_iter().chain(tested).collect()
    }
}
