use syn::spanned::Spanned;
use syn::{ExprCall, ExprMethodCall, ExprPath, Macro};

use code_model::{Building, Call, Callee, MacroCall, Region, Seat, TypeRef};
use lint_core::Syntax;
use lint_layout::{LineNumber, Name};

use crate::namespace::{Resolving, UsePath};

use super::Lowering;
use super::seat::{Built, Located};
use super::signature::Written;

#[derive(Debug)]
pub(super) struct CallRow {
    located: Located,
    callee: Callee,
    region: Region,
    subject: Option<TypeRef<Written>>,
}

impl Built for CallRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        let scope = self.located.scope();
        let subject = self.subject.map(|subject| Written::typed(subject, resolving, scope));
        self.located.call(building, Call::new(self.callee, Seat::new(self.region, subject)))
    }
}

#[derive(Clone, Copy, Debug)]
struct CallPath<'path> {
    path: &'path ExprPath,
}

impl CallPath<'_> {
    fn line(self) -> LineNumber {
        match self.path.path.segments.last() {
            Some(last) => LineNumber::from(last.ident.span()),
            None => LineNumber::from(self.path.span()),
        }
    }
}

impl Lowering<'_, '_> {
    pub(super) fn method_called(&mut self, call: &ExprMethodCall) {
        self.called(LineNumber::from(call.method.span()), Callee::Method(Name::from(&call.method)));
    }

    pub(super) fn path_called(&mut self, call: &ExprCall) {
        Syntax::expr_path(&call.func).into_iter().fold(self, |lowering, path| {
            let line = CallPath { path }.line();
            lowering.called(line, Callee::Path(UsePath::from(&path.path).written()));
            lowering
        });
    }

    pub(super) fn macro_called(&mut self, invoked: &Macro) {
        let line = LineNumber::from(invoked.path.span());
        self.called(line, Callee::Macro(MacroCall::from(UsePath::from(&invoked.path).written())));
    }

    fn called(&mut self, line: LineNumber, callee: Callee) {
        let located = self.here.located(line);
        let row = CallRow { located, callee, region: self.context.region(), subject: self.context.subject() };
        self.lowered.calls.push(row);
    }
}
