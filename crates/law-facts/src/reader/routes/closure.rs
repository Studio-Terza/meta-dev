use syn::{Expr, ExprClosure, ExprMethodCall, Stmt};

mod parameters;

use super::Routes;
use crate::law_path::LawPath;
use lint_layout::Name;
use parameters::Parameters;

#[derive(Debug)]
pub(crate) struct Closure<'routes, 'scope> {
    routes: &'routes Routes<'scope>,
    parameters: Parameters<'routes>,
}

impl<'routes, 'scope> Closure<'routes, 'scope> {
    pub(crate) fn resolved(routes: &'routes Routes<'scope>, closure: &ExprClosure) -> Option<LawPath> {
        let parameters = Parameters::declared(routes.scope(), &closure.inputs, routes.operand());
        Closure { routes, parameters }.body(&closure.body)
    }

    fn body(&self, body: &Expr) -> Option<LawPath> {
        match body {
            Expr::Block(block) => self.statements(&block.block.stmts),
            Expr::Paren(inner) => self.body(&inner.expr),
            Expr::MethodCall(call) => self.method(call),
            Expr::Call(call) => self.routes.called(&call.func),
            _ => None,
        }
    }

    fn statements(&self, statements: &[Stmt]) -> Option<LawPath> {
        statements.split_last().filter(|(_, rest)| rest.is_empty()).and_then(|(last, _)| self.statement(last))
    }

    fn statement(&self, statement: &Stmt) -> Option<LawPath> {
        match statement {
            Stmt::Expr(inner, None) => self.body(inner),
            _ => None,
        }
    }

    fn method(&self, call: &ExprMethodCall) -> Option<LawPath> {
        self.parameters.typed(&call.receiver).map(|owner| LawPath::new(owner, Name::from(&call.method)))
    }
}
