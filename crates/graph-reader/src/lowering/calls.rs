use syn::spanned::Spanned;
use syn::{ExprCall, ExprMacro, ExprMethodCall};

use graph::{Building, Call, Callee, ItemPath, MacroCall, Region};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use super::Lowering;
use super::places::PlaceId;
use super::seat::{Built, Footing, Located};

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: Callee,
    region: Region,
    place: Option<PlaceId>,
}

impl Built for CallRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let seat = footing.places().seat(self.region.joined(footing.outer()), self.place);
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

impl Lowering {
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

    pub(super) fn macro_called(&mut self, expression: &ExprMacro) {
        let invoked = &expression.mac;
        let called = CallPath { path: &invoked.path };
        self.called(Position::from(invoked.path.span()), Callee::Macro(MacroCall::from(called.written())));
    }

    fn called(&mut self, position: Position, callee: Callee) {
        let located = self.here.located(position);
        let row = CallRow { located, callee, region: self.context.region(), place: self.context.place() };
        self.lowered.rows.calls.push(row);
    }
}
