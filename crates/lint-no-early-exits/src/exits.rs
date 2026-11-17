use std::path::Path;
mod finding;

use syntax::Span;
use syntax::{ExprBreak, ExprContinue, ExprReturn, Macro};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Facts, Fold, Folding, Kind, Sweep, Visit};

use finding::Finding;

#[derive(Debug)]
pub struct Exits<'file> {
    file: &'file SourceFile,
}

impl<'file> Exits<'file> {
    fn record(&self, found: Vec<Violation<'file>>, span: Span, finding: Finding) -> Visit<Vec<Violation<'file>>> {
        let exit = Violation::new(Site::new(AsRef::<Path>::as_ref(self.file.head()), LineNumber::from(span)), finding);
        Visit::descending(found.into_iter().chain([exit]).collect())
    }
}

impl<'file> From<&'file SourceFile> for Exits<'file> {
    fn from(file: &'file SourceFile) -> Exits<'file> {
        Exits { file }
    }
}

impl<'file> Fold for Exits<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ExprReturn, Kind::ExprBreak, Kind::ExprContinue]
    }

    fn finished(&self, found: Self::Found) -> Self::Found {
        let modules = self.file.test_modules().iter();
        let tested = modules.flat_map(|module| Sweep::found(module, Box::new(Folding::from(Exits::from(module)))));
        found.into_iter().chain(tested).collect()
    }

    fn on_macro(&self, found: Self::Found, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::new(found, Descend::Expand)
    }

    fn on_expr_return(&self, found: Self::Found, expr: &ExprReturn, _: &Facts<'_, ExprReturn>) -> Visit<Self::Found> {
        self.record(found, expr.return_token.span, Finding::Return)
    }

    fn on_expr_break(&self, found: Self::Found, expr: &ExprBreak, _: &Facts<'_, ExprBreak>) -> Visit<Self::Found> {
        self.record(found, expr.break_token.span, Finding::Break)
    }

    fn on_expr_continue(
        &self,
        found: Self::Found,
        expr: &ExprContinue,
        _: &Facts<'_, ExprContinue>,
    ) -> Visit<Self::Found> {
        self.record(found, expr.continue_token.span, Finding::Continue)
    }
}
