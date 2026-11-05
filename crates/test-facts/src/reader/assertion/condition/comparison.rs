use syn::{BinOp, Expr, ExprBinary};

use super::super::super::trace::{Lowering, Trace};
use super::Condition;
use crate::boundary::Truth;
use crate::claim::{Accepted, Claim, Exclusion, Order, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Relational {
    Each,
    Disjunction,
    Equal,
    Unequal,
    Ordered(Order),
    Other,
}

impl Relational {
    const COMPLEMENTS: [(Order, Order); 4] = [
        (Order::Below, Order::AtLeast),
        (Order::AtMost, Order::Above),
        (Order::Above, Order::AtMost),
        (Order::AtLeast, Order::Below),
    ];

    fn read(operator: BinOp) -> Relational {
        match operator {
            BinOp::And(_) => Relational::Each,
            BinOp::Or(_) => Relational::Disjunction,
            BinOp::Eq(_) => Relational::Equal,
            BinOp::Ne(_) => Relational::Unequal,
            BinOp::Lt(_) => Relational::Ordered(Order::Below),
            BinOp::Le(_) => Relational::Ordered(Order::AtMost),
            BinOp::Gt(_) => Relational::Ordered(Order::Above),
            BinOp::Ge(_) => Relational::Ordered(Order::AtLeast),
            _ => Relational::Other,
        }
    }

    fn under(self, holds: Truth) -> Relational {
        match holds {
            Truth::Holds => self,
            Truth::Fails => self.negated(),
        }
    }

    fn negated(self) -> Relational {
        match self {
            Relational::Each | Relational::Other => Relational::Other,
            Relational::Disjunction => Relational::Each,
            Relational::Equal => Relational::Unequal,
            Relational::Unequal => Relational::Equal,
            Relational::Ordered(order) => Self::COMPLEMENTS
                .iter()
                .find(|(held, _)| *held == order)
                .map_or(Relational::Other, |(_, complement)| Relational::Ordered(*complement)),
        }
    }

    fn claims(self, comparison: &Comparison<'_, '_, '_>) -> Vec<Claim> {
        match self {
            Relational::Each => Condition::held(comparison.trace, &comparison.binary.left, comparison.holds)
                .into_iter()
                .chain(Condition::held(comparison.trace, &comparison.binary.right, comparison.holds))
                .collect(),
            Relational::Equal => comparison.trace.stated(
                &comparison.binary.left,
                Accepted::equal(Lowering::term_of(comparison.trace, &comparison.binary.right)),
            ),
            Relational::Unequal => comparison.trace.stated(
                &comparison.binary.left,
                Accepted::Excluding(Exclusion::from(Lowering::term_of(comparison.trace, &comparison.binary.right))),
            ),
            Relational::Ordered(order) => comparison.ordered(order),
            Relational::Disjunction | Relational::Other => {
                comparison.trace.stated(comparison.condition, Accepted::equal(Term::Truth(comparison.holds)))
            }
        }
    }
}

pub(crate) struct Comparison<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    binary: &'trace ExprBinary,
    condition: &'trace Expr,
    holds: Truth,
}

impl<'trace, 'body, 'file> Comparison<'trace, 'body, 'file> {
    pub(crate) fn new(
        trace: &'trace Trace<'body, 'file>,
        binary: &'trace ExprBinary,
        condition: &'trace Expr,
        holds: Truth,
    ) -> Comparison<'trace, 'body, 'file> {
        Comparison { trace, binary, condition, holds }
    }

    pub(crate) fn claims(&self) -> Vec<Claim> {
        Relational::read(self.binary.op).under(self.holds).claims(self)
    }

    fn ordered(&self, order: Order) -> Vec<Claim> {
        let left = Lowering::term_of(self.trace, &self.binary.left);
        left.closed().map_or_else(
            || {
                self.trace.stated(
                    &self.binary.left,
                    Accepted::ordered(order, Lowering::term_of(self.trace, &self.binary.right)),
                )
            },
            |_| self.trace.stated(&self.binary.right, Accepted::ordered(order.reversed(), left.clone())),
        )
    }
}
