mod comparison;
mod predicates;
mod shape;

use syn::{Expr, ExprUnary, UnOp};

use super::super::trace::Trace;
use crate::boundary::Truth;
use crate::claim::{Accepted, Claim, Term};
use comparison::Comparison;
use predicates::Predicates;
use shape::Shape;

pub(crate) struct Condition<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    holds: Truth,
}

impl<'trace, 'body, 'file> Condition<'trace, 'body, 'file> {
    pub(crate) fn claims(trace: &'trace Trace<'body, 'file>, condition: &Expr) -> Vec<Claim> {
        Condition { trace, holds: Truth::Holds }.read(condition)
    }

    pub(crate) fn held(trace: &'trace Trace<'body, 'file>, condition: &Expr, holds: Truth) -> Vec<Claim> {
        Condition { trace, holds }.read(condition)
    }

    fn read(&self, condition: &Expr) -> Vec<Claim> {
        match condition {
            Expr::Paren(inner) => self.read(&inner.expr),
            Expr::Unary(ExprUnary { op: UnOp::Not(_), expr, .. }) => {
                Condition::held(self.trace, expr, self.holds.negated())
            }
            Expr::Binary(binary) => Comparison::new(self.trace, binary, condition, self.holds).claims(),
            Expr::Macro(inner) => Shape::matched(self.trace, &inner.mac, self.holds)
                .unwrap_or_else(|| self.trace.stated(condition, Accepted::equal(Term::Truth(self.holds)))),
            Expr::MethodCall(call) => Predicates::claims(self.trace, call, self.holds)
                .unwrap_or_else(|| self.trace.stated(condition, Accepted::equal(Term::Truth(self.holds)))),
            _ => self.trace.stated(condition, Accepted::equal(Term::Truth(self.holds))),
        }
    }
}
