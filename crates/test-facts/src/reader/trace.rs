mod callees;
mod constants;
mod evaluation;
mod extent;
mod iterations;
mod leaves;
mod locals;
mod tautology;

use newtype::Counter;
use syn::{Expr, ExprLit, Lit, Macro};

use lint_core::{Macros, Syntax};

use super::scope::{Place, Scope};
use crate::check::{Check, Domain, Relation, Values};
use callees::Callees;
pub(crate) use constants::Constants;
pub(crate) use evaluation::Evaluation;
pub(crate) use extent::Extent;
pub(crate) use iterations::Iterations;
pub(crate) use leaves::Leaves;
pub(crate) use locals::Locals;
use tautology::Tautology;

#[derive(Debug)]
pub(crate) struct Trace<'body, 'file> {
    scope: &'body Scope<'file>,
    locals: &'body Locals<'file>,
    place: &'body Place,
}

impl<'body, 'file> Trace<'body, 'file> {
    const SIDES_LENGTH: usize = 2;

    pub(crate) fn new(
        scope: &'body Scope<'file>,
        locals: &'body Locals<'file>,
        place: &'body Place,
    ) -> Trace<'body, 'file> {
        Trace { scope, locals, place }
    }

    pub(crate) fn compared(&self, relation: Relation, sides: &[&Expr], domain: Domain) -> Vec<Check> {
        let relation = self.vacuous(sides).unwrap_or(relation);
        let unbooleaned: Vec<&Expr> = sides.iter().copied().filter(|side| Self::boolean(side).is_none()).collect();
        let measured = Self::distinct(&unbooleaned);
        let values = Values::total(measured.iter().map(|side| Leaves::counted(self, side)));
        vec![Check::new(relation, values, domain, Callees::within(self, sides))]
    }

    fn vacuous(&self, sides: &[&Expr]) -> Option<Relation> {
        Tautology::among(self, sides).or_else(|| sides.iter().copied().find_map(Self::boolean).map(|_| Relation::Holds))
    }

    pub(crate) fn operands(&self, relation: Relation, invocation: &Macro, domain: Domain) -> Vec<Check> {
        let arguments = Macros::arguments(invocation);
        let sides: Vec<&Expr> = arguments.iter().take(Self::SIDES_LENGTH).collect();
        self.compared(relation, &sides, domain)
    }

    pub(crate) fn distinct<'side, Item: PartialEq>(sides: &[&'side Item]) -> Vec<&'side Item> {
        sides
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, side)| sides.iter().position(|other| other == side) == Some(*index))
            .map(|(_, side)| side)
            .collect()
    }

    pub(crate) fn bound(&self, expression: &Expr, constants: Constants) -> Option<&'file Expr> {
        Locals::named(expression).and_then(|name| self.locals.bound(&name)).or_else(|| {
            Syntax::expr_path(expression).and_then(|path| constants.looked_up(self.scope, &path.path, self.place))
        })
    }

    fn boolean(side: &Expr) -> Option<&Expr> {
        match side {
            Expr::Lit(ExprLit { lit: Lit::Bool(_), .. }) => Some(side),
            _ => None,
        }
    }
}
