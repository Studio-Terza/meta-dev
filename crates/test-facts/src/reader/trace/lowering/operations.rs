use syn::{BinOp, Lit, Member, Path};

use lint_layout::{Casing, Name};

use crate::boundary::{Position, Quantity, Spelling, Truth};
use crate::claim::{Callee, Operator, Term};

pub(crate) struct Operations;

impl Operations {
    pub(crate) fn literal(literal: &Lit) -> Term {
        match literal {
            Lit::Int(integer) => integer
                .base10_parse()
                .map_or_else(|_| Term::Spelled(Spelling::written(literal)), |value| Term::Number(Quantity::new(value))),
            Lit::Bool(flag) => Term::Truth(Truth::judged(flag.value)),
            _ => Term::Spelled(Spelling::written(literal)),
        }
    }

    pub(crate) fn member(member: &Member) -> Name {
        match member {
            Member::Named(name) => Name::from(name),
            Member::Unnamed(index) => Name::from(Position::counted(usize::try_from(index.index).unwrap_or_default())),
        }
    }

    pub(crate) fn operator(operator: BinOp) -> Option<Operator> {
        match operator {
            BinOp::Add(_) | BinOp::AddAssign(_) => Some(Operator::Add),
            BinOp::Sub(_) | BinOp::SubAssign(_) => Some(Operator::Subtract),
            BinOp::Mul(_) | BinOp::MulAssign(_) => Some(Operator::Multiply),
            BinOp::Div(_) | BinOp::DivAssign(_) => Some(Operator::Divide),
            BinOp::Rem(_) | BinOp::RemAssign(_) => Some(Operator::Remainder),
            BinOp::And(_) => Some(Operator::Both),
            BinOp::Or(_) => Some(Operator::Either),
            BinOp::Eq(_) => Some(Operator::Equal),
            BinOp::Ne(_) => Some(Operator::Unequal),
            BinOp::Lt(_) => Some(Operator::Below),
            BinOp::Le(_) => Some(Operator::AtMost),
            BinOp::Gt(_) => Some(Operator::Above),
            BinOp::Ge(_) => Some(Operator::AtLeast),
            _ => None,
        }
    }

    pub(crate) fn constructed(path: &Path) -> Option<Name> {
        path.segments.last().map(|last| Name::from(&last.ident)).filter(|last| last.casing() == Casing::Camel)
    }

    pub(crate) fn constructor(path: &Path, arguments: Vec<Term>) -> Term {
        Operations::constructed(path)
            .map_or_else(|| Term::Opaque(Spelling::written(path)), |name| Callee::Constructor(name).folded(arguments))
    }
}
