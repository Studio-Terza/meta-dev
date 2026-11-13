use syn::spanned::Spanned;
use syn::{ExprCall, ExprMethodCall, Macro};

use code_model::{Building, Call, Callee, ItemPath, MacroCall, Region};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use super::Lowering;
use super::seat::{Built, Footing, Located};
use super::subjects::SubjectId;

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: Callee,
    region: Region,
    subject: Option<SubjectId>,
}

impl Built for CallRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let seat = footing.subjects().seat(self.region.joined(footing.outer()), self.subject);
        self.located.call(footing, building, Call::new(self.callee, seat))
    }
}

#[derive(Clone, Copy, Debug)]
struct CallPath<'path> {
    path: &'path syn::Path,
}

impl CallPath<'_> {
    fn position(self) -> Position {
        match self.path.segments.last() {
            Some(last) => Position::from(last.ident.span()),
            None => Position::from(self.path.span()),
        }
    }

    fn written(self) -> ItemPath {
        self.path.segments.iter().map(|segment| Name::from(&segment.ident)).collect()
    }
}

impl Lowering<'_> {
    pub(super) fn method_called(&mut self, call: &ExprMethodCall) {
        self.called(Position::from(call.method.span()), Callee::Method(Name::from(&call.method)));
    }

    pub(super) fn path_called(&mut self, call: &ExprCall) {
        Syntax::expr_path(&call.func).into_iter().fold(self, |lowering, path| {
            let called = CallPath { path: &path.path };
            lowering.called(called.position(), Callee::Path(called.written()));
            lowering
        });
    }

    pub(super) fn macro_called(&mut self, invoked: &Macro) {
        let called = CallPath { path: &invoked.path };
        self.called(Position::from(invoked.path.span()), Callee::Macro(MacroCall::from(called.written())));
    }

    fn called(&mut self, position: Position, callee: Callee) {
        let located = self.here.located(position);
        let row = CallRow { located, callee, region: self.context.region(), subject: self.context.subject() };
        self.lowered.rows.calls.push(row);
    }
}
