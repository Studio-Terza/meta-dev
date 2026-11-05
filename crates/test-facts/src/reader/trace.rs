mod iterations;
mod leaves;
mod locals;

use newtype::Counter;
use syn::{Expr, ExprLit, Lit};

use super::scope::Scope;
use crate::check::{Check, Domain, Relation, Values};
pub(crate) use iterations::Iterations;
pub(crate) use leaves::Leaves;
pub(crate) use locals::Locals;

#[derive(Debug)]
pub(crate) struct Trace<'body, 'file> {
    scope: &'body Scope<'file>,
    locals: &'body Locals<'file>,
}

impl<'body, 'file> Trace<'body, 'file> {
    pub(crate) fn new(scope: &'body Scope<'file>, locals: &'body Locals<'file>) -> Trace<'body, 'file> {
        Trace { scope, locals }
    }

    pub(crate) fn compared(&self, relation: Relation, sides: &[&Expr], domain: Domain) -> Vec<Check> {
        let relation =
            sides.iter().copied().filter_map(Self::boolean).map(|_| Relation::Holds).next().unwrap_or(relation);
        let measured: Vec<&Expr> = sides.iter().copied().filter(|side| Self::boolean(side).is_none()).collect();
        let values = Values::total(measured.iter().map(|side| Leaves::counted(self, side)));
        let domain = measured.iter().map(|side| Iterations::within(self, side)).fold(domain, Domain::max_with);
        vec![Check::new(relation, values, domain)]
    }

    fn bound(&self, expression: &Expr) -> Option<&'file Expr> {
        let name = Locals::named(expression)?;
        self.locals.bound(&name).or_else(|| self.scope.constant(&name))
    }

    fn boolean(side: &Expr) -> Option<&Expr> {
        match side {
            Expr::Lit(ExprLit { lit: Lit::Bool(_), .. }) => Some(side),
            _ => None,
        }
    }
}
