use std::path::Path;
mod finding;

use syntax::Span;
use syntax::{ExprBreak, ExprContinue, ExprReturn, Macro};

use lint_core::{Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Descend, Facts, Kind, Sweep};

use finding::Finding;

#[derive(Debug)]
pub struct Exits<'file> {
    file: &'file SourceFile,
}

impl<'file> Exits<'file> {
    fn record(&self, span: Span, finding: Finding) -> Contributed<(), Vec<Violation<'file>>> {
        let site = Site::new(AsRef::<Path>::as_ref(self.file.head()), LineNumber::from(span));
        Contributed::parted(vec![Violation::new(site, finding)])
    }
}

impl<'file> From<&'file SourceFile> for Exits<'file> {
    fn from(file: &'file SourceFile) -> Exits<'file> {
        Exits { file }
    }
}

impl<'file> Contribute for Exits<'file> {
    type Open = ();
    type Part = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Macro, Kind::ExprReturn, Kind::ExprBreak, Kind::ExprContinue]
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        Contributed::descending(Descend::Expand)
    }

    fn on_expr_return(
        &self,
        expr: &ExprReturn,
        _: &Facts<'_, ExprReturn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        self.record(expr.return_token.span, Finding::Return)
    }

    fn on_expr_break(
        &self,
        expr: &ExprBreak,
        _: &Facts<'_, ExprBreak>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        self.record(expr.break_token.span, Finding::Break)
    }

    fn on_expr_continue(
        &self,
        expr: &ExprContinue,
        _: &Facts<'_, ExprContinue>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Vec<Violation<'file>>> {
        self.record(expr.continue_token.span, Finding::Continue)
    }
}

impl<'file> Concludes<'file> for Exits<'file> {
    fn violations(self, (): (), found: Vec<Violation<'file>>) -> Vec<Violation<'file>> {
        let tested = self
            .file
            .test_modules()
            .iter()
            .flat_map(|module| Sweep::found(module, Contributing::new(Exits::from(module))));
        found.into_iter().chain(tested).collect()
    }
}
