use std::collections::BTreeSet;

use syn::token::Minus;
use syn::{BinOp, Expr, ExprBinary, ExprLit, ExprUnary, Lit, UnOp};

use lint_layout::Name;

use super::{Constants, Locals, Trace};
use crate::boundary::{Quantity, Truth};

pub(crate) struct Evaluation<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    followed: BTreeSet<Name>,
}

impl<'trace, 'body, 'file> Evaluation<'trace, 'body, 'file> {
    pub(crate) fn quantity_of(trace: &'trace Trace<'body, 'file>, expression: &Expr) -> Option<Quantity> {
        Evaluation { trace, followed: BTreeSet::new() }.quantity(expression)
    }

    pub(crate) fn condition_of(trace: &'trace Trace<'body, 'file>, expression: &Expr) -> Option<Truth> {
        Evaluation { trace, followed: BTreeSet::new() }.condition(expression)
    }

    pub(crate) fn quantity(&mut self, expression: &Expr) -> Option<Quantity> {
        match expression {
            Expr::Lit(ExprLit { lit: Lit::Int(literal), .. }) => literal.base10_parse().ok().map(Quantity::new),
            Expr::Paren(inner) => self.quantity(&inner.expr),
            Expr::Cast(cast) => self.quantity(&cast.expr),
            Expr::Unary(ExprUnary { op: UnOp::Neg(_), expr, .. }) => {
                self.quantity(expr).and_then(|inner| Quantity::ZERO.applied(&BinOp::Sub(Minus::default()), inner))
            }
            Expr::Binary(binary) => self.calculated(binary),
            Expr::Path(_) => self.following(expression, Self::quantity),
            _ => None,
        }
    }

    pub(crate) fn calculated(&mut self, binary: &ExprBinary) -> Option<Quantity> {
        let left = self.quantity(&binary.left)?;
        let right = self.quantity(&binary.right)?;
        left.applied(&binary.op, right)
    }

    pub(crate) fn condition(&mut self, expression: &Expr) -> Option<Truth> {
        match expression {
            Expr::Lit(ExprLit { lit: Lit::Bool(literal), .. }) => Some(Truth::judged(literal.value)),
            Expr::Paren(inner) => self.condition(&inner.expr),
            Expr::Unary(ExprUnary { op: UnOp::Not(_), expr, .. }) => self.condition(expr).map(Truth::negated),
            Expr::Binary(binary) => self.ordered(binary),
            Expr::Path(_) => self.following(expression, Self::condition),
            _ => None,
        }
    }

    pub(crate) fn ordered(&mut self, binary: &ExprBinary) -> Option<Truth> {
        let left = self.quantity(&binary.left)?;
        let right = self.quantity(&binary.right)?;
        left.tested(&binary.op, right)
    }

    pub(crate) fn following<Value>(
        &mut self,
        expression: &Expr,
        measure: fn(&mut Self, &Expr) -> Option<Value>,
    ) -> Option<Value> {
        let guard = Locals::named(expression);
        let measured = guard
            .as_ref()
            .map_or(Some(()), |name| self.followed.insert(name.clone()).then_some(()))
            .and_then(|()| self.trace.bound(expression, Constants::Followed))
            .and_then(|bound| measure(self, bound));
        self.followed.retain(|followed| Some(followed) != guard.as_ref());
        measured
    }
}
