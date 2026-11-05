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
            BinOp::Add(_) => Some(Operator::Add),
            BinOp::Sub(_) => Some(Operator::Subtract),
            BinOp::Mul(_) => Some(Operator::Multiply),
            BinOp::Div(_) => Some(Operator::Divide),
            BinOp::Rem(_) => Some(Operator::Remainder),
            BinOp::And(_) => Some(Operator::Both),
            BinOp::Or(_) => Some(Operator::Either),
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
