use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMethodCall, Macro};

use lint_core::{Macros, Syntax};
use lint_layout::Name;

use super::Trace;
use crate::check::Calls;

pub(crate) struct Callees<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    found: Vec<Name>,
}

impl<'trace, 'body, 'file> Callees<'trace, 'body, 'file> {
    pub(crate) fn within(trace: &'trace Trace<'body, 'file>, sides: &[&Expr]) -> Calls {
        let callees = sides.iter().fold(Callees { trace, found: Vec::new() }, |mut callees, side| {
            callees.visit_expr(side);
            callees
        });
        Calls::naming(callees.found)
    }
}

impl<'ast> Visit<'ast> for Callees<'_, '_, '_> {
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.found
            .extend(Syntax::expr_path(&call.func).map(|path| self.trace.scope.called(&path.path, self.trace.place)));
        visit::visit_expr_call(self, call);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.found.push(Name::from(&call.method));
        visit::visit_expr_method_call(self, call);
    }
}
