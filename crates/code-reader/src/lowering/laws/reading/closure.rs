use std::collections::BTreeMap;

use syn::punctuated::Punctuated;
use syn::{Expr, ExprClosure, ExprMethodCall, ExprPath, Pat, PatType, Stmt, Token};

use lint_layout::Name;

use super::Checking;
use crate::lowering::laws::{LoweredOperation, LoweredOwner};
use crate::lowering::signature::InScope;

#[derive(Debug)]
pub(super) struct Closure<'scope, 'owner> {
    checking: Checking<'scope, 'owner>,
    parameters: Parameters,
}

#[derive(Debug)]
struct Parameter {
    name: Name,
    typed: Option<LoweredOwner>,
}

#[derive(Debug)]
struct Parameters(BTreeMap<Name, Option<LoweredOwner>>);

impl<'scope, 'owner> Closure<'scope, 'owner> {
    pub(super) fn resolved(checking: Checking<'scope, 'owner>, closure: &ExprClosure) -> Option<LoweredOperation> {
        let parameters = Parameters::declared(checking.scope(), &closure.inputs);
        Closure { checking, parameters }.body(&closure.body)
    }

    fn body(&self, body: &Expr) -> Option<LoweredOperation> {
        match body {
            Expr::Block(block) => self.statements(&block.block.stmts),
            Expr::Paren(inner) => self.body(&inner.expr),
            Expr::MethodCall(call) => self.method(call),
            Expr::Call(call) => self.checking.called(&call.func),
            _ => None,
        }
    }

    fn statements(&self, statements: &[Stmt]) -> Option<LoweredOperation> {
        let (last, rest) = statements.split_last()?;
        rest.first().map_or_else(|| self.statement(last), |_| None)
    }

    fn statement(&self, statement: &Stmt) -> Option<LoweredOperation> {
        match statement {
            Stmt::Expr(inner, None) => self.body(inner),
            _ => None,
        }
    }

    fn method(&self, call: &ExprMethodCall) -> Option<LoweredOperation> {
        let owner = self.parameters.typed(&call.receiver, self.checking.fallback())?;
        Some(LoweredOperation { owner, member: Name::from(&call.method) })
    }
}

impl Parameter {
    fn written(scope: &InScope, pattern: &Pat) -> Option<Parameter> {
        match pattern {
            Pat::Ident(named) => Some(Parameter { name: Name::from(&named.ident), typed: None }),
            Pat::Type(typed) => Self::annotated(scope, typed),
            _ => None,
        }
    }

    fn annotated(scope: &InScope, typed: &PatType) -> Option<Parameter> {
        match &*typed.pat {
            Pat::Ident(named) => Some(Parameter {
                name: Name::from(&named.ident),
                typed: Some(LoweredOwner::Typed(scope.lowered(&typed.ty))),
            }),
            _ => None,
        }
    }
}

impl Parameters {
    fn declared(scope: &InScope, inputs: &Punctuated<Pat, Token![,]>) -> Parameters {
        Parameters(
            inputs
                .iter()
                .filter_map(|pattern| Parameter::written(scope, pattern))
                .map(|parameter| (parameter.name, parameter.typed))
                .collect(),
        )
    }

    fn typed(&self, receiver: &Expr, fallback: Option<&LoweredOwner>) -> Option<LoweredOwner> {
        match receiver {
            Expr::Path(found) => self.named(found, fallback),
            Expr::Paren(inner) => self.typed(&inner.expr, fallback),
            Expr::Reference(reference) => self.typed(&reference.expr, fallback),
            Expr::Unary(unary) => self.typed(&unary.expr, fallback),
            _ => None,
        }
    }

    fn named(&self, found: &ExprPath, fallback: Option<&LoweredOwner>) -> Option<LoweredOwner> {
        let typed = self.0.get(&Name::from(found.path.get_ident()?))?;
        typed.clone().or_else(|| fallback.cloned())
    }
}
