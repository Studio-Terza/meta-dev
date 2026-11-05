use std::cmp::Ordering;

use syn::BinOp;

use lint_layout::Name;
use newtype::{Arbitrary, Display, Newtype};

use crate::claim::{Cases, Operator};

type Arithmetic = fn(i128, i128) -> Option<i128>;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Quantity(i128);

impl Quantity {
    pub(crate) const ZERO: Quantity = Quantity::new(0);
    pub(crate) const UNIT: Quantity = Quantity::new(1);
    pub(crate) const LEAST: Quantity = Quantity::new(i128::MIN);
    pub(crate) const GREATEST: Quantity = Quantity::new(i128::MAX);
    const ARITHMETIC: [(Operator, Arithmetic); 5] = [
        (Operator::Add, i128::checked_add),
        (Operator::Subtract, i128::checked_sub),
        (Operator::Multiply, i128::checked_mul),
        (Operator::Divide, i128::checked_div),
        (Operator::Remainder, i128::checked_rem),
    ];

    pub(crate) fn cases(self) -> Option<Cases> {
        usize::try_from(self.into_inner()).ok().map(Cases::new)
    }

    pub(crate) fn combined(self, operator: Operator, other: Quantity) -> Option<Quantity> {
        Self::ARITHMETIC
            .iter()
            .find(|(arithmetic, _)| *arithmetic == operator)
            .and_then(|(_, apply)| apply(self.into_inner(), other.into_inner()))
            .map(Quantity::new)
    }

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

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Spelling(String);

impl Spelling {
    pub(crate) fn written(tokens: &impl quote::ToTokens) -> Spelling {
        Spelling(tokens.to_token_stream().to_string())
    }
}

impl std::fmt::Display for Spelling {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Position(usize);

impl Position {
    pub(crate) fn named(name: &Name) -> Option<Position> {
        name.to_string().parse().ok().map(Position)
    }

    pub(crate) fn counted(index: usize) -> Position {
        Position(index)
    }

    pub(crate) fn numbered(index: Quantity) -> Option<Position> {
        usize::try_from(index.into_inner()).ok().map(Position)
    }

    pub(crate) fn picked<Item>(self, items: &[Item]) -> Option<&Item> {
        items.get(self.0)
    }
}

impl From<Position> for Name {
    fn from(position: Position) -> Name {
        Name::from(position.0.to_string().as_str())
    }
}

impl std::fmt::Display for Truth {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Truth::Holds => formatter.write_str("true"),
            Truth::Fails => formatter.write_str("false"),
        }
    }
}
