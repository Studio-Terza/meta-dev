use std::cmp::Ordering;

use syn::BinOp;

use newtype::{Arbitrary, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Quantity(i128);

impl Quantity {
    pub(crate) const ZERO: Quantity = Quantity::new(0);
    pub(crate) const UNIT: Quantity = Quantity::new(1);

    pub(crate) fn applied(self, operator: &BinOp, other: Quantity) -> Option<Quantity> {
        match operator {
            BinOp::Add(_) => self.into_inner().checked_add(other.into_inner()).map(Quantity::new),
            BinOp::Sub(_) => self.into_inner().checked_sub(other.into_inner()).map(Quantity::new),
            BinOp::Mul(_) => self.into_inner().checked_mul(other.into_inner()).map(Quantity::new),
            BinOp::Div(_) => self.into_inner().checked_div(other.into_inner()).map(Quantity::new),
            BinOp::Rem(_) => self.into_inner().checked_rem(other.into_inner()).map(Quantity::new),
            _ => None,
        }
    }

    pub(crate) fn tested(self, operator: &BinOp, other: Quantity) -> Option<Truth> {
        Self::judged(operator, self.cmp(&other))
    }

    fn judged(operator: &BinOp, order: Ordering) -> Option<Truth> {
        match operator {
            BinOp::Eq(_) => Some(Truth::judged(order.is_eq())),
            BinOp::Ne(_) => Some(Truth::judged(order.is_ne())),
            BinOp::Lt(_) => Some(Truth::judged(order.is_lt())),
            BinOp::Le(_) => Some(Truth::judged(order.is_le())),
            BinOp::Gt(_) => Some(Truth::judged(order.is_gt())),
            BinOp::Ge(_) => Some(Truth::judged(order.is_ge())),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Truth {
    Fails,
    Holds,
}

impl Truth {
    const BY_FLAG: [Truth; 2] = [Truth::Fails, Truth::Holds];

    pub(crate) fn judged(held: bool) -> Truth {
        Self::BY_FLAG.get(usize::from(held)).copied().unwrap_or(Truth::Fails)
    }

    pub(crate) fn negated(self) -> Truth {
        match self {
            Truth::Holds => Truth::Fails,
            Truth::Fails => Truth::Holds,
        }
    }
}
