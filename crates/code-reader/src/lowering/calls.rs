use syn::spanned::Spanned;
use syn::{ExprCall, ExprMethodCall, Macro};

use code_model::{Building, Call, Callee, ItemPath, MacroCall, Region};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};

use super::Lowering;
use super::seat::Located;
use super::subjects::{SubjectId, Subjects};

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: Callee,
    region: Region,
    subject: Option<SubjectId>,
}

impl CallRow {
    pub(super) fn built(self, subjects: &Subjects, building: Building) -> Building {
        self.located.call(building, Call::new(self.callee, subjects.seat(self.region, self.subject)))
    }
}

#[derive(Clone, Copy, Debug)]
struct CallPath<'path> {
    path: &'path syn::Path,
}

impl CallPath<'_> {
    fn line(self) -> LineNumber {
        match self.path.segments.last() {
            Some(last) => LineNumber::from(last.ident.span()),
            None => LineNumber::from(self.path.span()),
        }
    }

    fn written(self) -> ItemPath {
        self.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect()
    }
}

impl Lowering<'_, '_> {
    pub(super) fn method_called(&mut self, call: &ExprMethodCall) {
        self.called(LineNumber::from(call.method.span()), Callee::Method(Name::from(&call.method)));
    }

    pub(super) fn path_called(&mut self, call: &ExprCall) {
        Syntax::expr_path(&call.func).into_iter().fold(self, |lowering, path| {
            let called = CallPath { path: &path.path };
            lowering.called(called.line(), Callee::Path(called.written()));
            lowering
        });
    }

    pub(super) fn macro_called(&mut self, invoked: &Macro) {
        let called = CallPath { path: &invoked.path };
        self.called(LineNumber::from(invoked.path.span()), Callee::Macro(MacroCall::from(called.written())));
    }

    fn called(&mut self, line: LineNumber, callee: Callee) {
        let located = self.here.located(line);
        let row = CallRow { located, callee, region: self.context.region(), subject: self.context.subject() };
        self.lowered.calls.push(row);
    }
}
