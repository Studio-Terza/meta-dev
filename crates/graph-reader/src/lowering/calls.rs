use super::{Frame, Grain, Grains};
use syntax::Spanned;
use syntax::{ExprCall, ExprMacro, ExprMethodCall};

use graph::{Call, Callee, ItemPath, MacroCall, ModelPart, Region};
use lint_core::Syntax;
use lint_layout::{Name, Position};

use super::places::PlaceId;
use super::seat::{Footing, Located, Lowers};

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: Callee,
    region: Region,
    place: Option<PlaceId>,
}

impl Lowers for CallRow {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        let seat = footing.places().seat(self.region.joined(footing.outer()), self.place.as_ref());
        self.located.call(footing, Call::new(self.callee, seat))
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

impl Frame<'_> {
    pub(super) fn method_called(&self, call: &ExprMethodCall) -> Grains {
        self.called(Position::from(call.method.span()), Callee::Method(Name::from(&call.method)))
    }

    pub(super) fn path_called(&self, call: &ExprCall) -> Grains {
        Syntax::expr_path(&call.func).map_or_else(Grains::default, |path| {
            let called = CallPath { path: &path.path };
            self.called(called.position(), Callee::Path(called.written()))
        })
    }

    pub(super) fn macro_called(&self, expression: &ExprMacro) -> Grains {
        let invoked = &expression.mac;
        let called = CallPath { path: &invoked.path };
        self.called(Position::from(invoked.path.span()), Callee::Macro(MacroCall::from(called.written())))
    }

    fn called(&self, position: Position, callee: Callee) -> Grains {
        let located = self.here.located(position);
        let row = CallRow { located, callee, region: self.context.region(), place: self.context.place() };
        Grains::from(Grain::Call(row))
    }
}
