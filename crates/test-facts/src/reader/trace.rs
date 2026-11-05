mod constants;
mod iterations;
mod leaves;
mod locals;

use newtype::Counter;
use syn::{Expr, ExprLit, Lit, Macro};

use lint_core::Macros;

use super::scope::Scope;
use crate::check::{Check, Domain, Relation, Values};
pub(crate) use constants::Constants;
pub(crate) use iterations::Iterations;
pub(crate) use leaves::Leaves;
pub(crate) use locals::Locals;

#[derive(Debug)]
pub(crate) struct Trace<'body, 'file> {
    scope: &'body Scope<'file>,
    locals: &'body Locals<'file>,
}

impl<'body, 'file> Trace<'body, 'file> {
    const SIDES_LENGTH: usize = 2;

    pub(crate) fn new(scope: &'body Scope<'file>, locals: &'body Locals<'file>) -> Trace<'body, 'file> {
        Trace { scope, locals }
    }

    pub(crate) fn compared(&self, relation: Relation, sides: &[&Expr], domain: Domain) -> Vec<Check> {
        let relation =
            sides.iter().copied().filter_map(Self::boolean).map(|_| Relation::Holds).next().unwrap_or(relation);
        let unbooleaned: Vec<&Expr> = sides.iter().copied().filter(|side| Self::boolean(side).is_none()).collect();
        let measured = Self::distinct(&unbooleaned);
        let values = Values::total(measured.iter().map(|side| Leaves::counted(self, side)));
        vec![Check::new(relation, values, domain)]
    }

    pub(crate) fn operands(&self, relation: Relation, invocation: &Macro, domain: Domain) -> Vec<Check> {
        let arguments = Macros::arguments(invocation);
        let sides: Vec<&Expr> = arguments.iter().take(Self::SIDES_LENGTH).collect();
        self.compared(relation, &sides, domain)
    }

    fn distinct<'side>(sides: &[&'side Expr]) -> Vec<&'side Expr> {
        sides
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, side)| sides.iter().position(|other| other == side) == Some(*index))
            .map(|(_, side)| side)
            .collect()
    }

    fn bound(&self, expression: &Expr, constants: Constants) -> Option<&'file Expr> {
        Locals::named(expression)
            .and_then(|name| self.locals.bound(&name).or_else(|| constants.looked_up(self.scope, &name)))
    }

    fn boolean(side: &Expr) -> Option<&Expr> {
        match side {
            Expr::Lit(ExprLit { lit: Lit::Bool(_), .. }) => Some(side),
            _ => None,
        }
    }
}
