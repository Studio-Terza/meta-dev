use syntax::Spanned;
use syntax::{ExprCall, ExprMacro, ExprMethodCall};

use graph::{Building, Call, Callee, ItemPath, MacroCall, Region};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use super::Lowering;
use super::places::PlaceId;
use super::seat::{Built, Footing, Located};
use lint_sweep::Address;

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
    path: &'path syntax::Path,
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

impl<'read> Lowering<'read> {
    pub(super) fn method_called(self: Box<Self>, call: &ExprMethodCall) -> Box<Lowering<'read>> {
        self.called(Position::from(call.method.span()), Callee::Method(Name::from(&call.method)))
    }

    pub(super) fn path_called(mut self: Box<Self>, call: &ExprCall) -> Box<Lowering<'read>> {
        self.calling = Syntax::expr_path(&call.func).map(Address::from);
        Syntax::expr_path(&call.func).into_iter().fold(self, |lowering, path| {
            let called = CallPath { path: &path.path };
            lowering.called(called.position(), Callee::Path(called.written()))
        })
    }

    pub(super) fn macro_called(self: Box<Self>, expression: &ExprMacro) -> Box<Lowering<'read>> {
        let invoked = &expression.mac;
        let called = CallPath { path: &invoked.path };
        self.called(Position::from(invoked.path.span()), Callee::Macro(MacroCall::from(called.written())))
    }

    fn called(mut self: Box<Self>, position: Position, callee: Callee) -> Box<Lowering<'read>> {
        let located = self.here.located(position);
        let row = CallRow { located, callee, region: self.context.region(), place: self.context.place() };
        self.lowered.rows.calls.push(row);
        self
    }
}
