use syn::{Expr, Lit};

use lint_layout::Name;

use super::super::pinning::Pinning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Plain,
    Format,
}

#[derive(Debug)]
pub struct Literal;

impl Literal {
    const LINE_BREAK: char = '\n';

    pub fn pinning(literal: &Lit, reading: Reading) -> Option<Pinning> {
        Self::read(literal).and_then(|pinning| Self::shown(pinning, reading))
    }

    fn read(literal: &Lit) -> Option<Pinning> {
        match literal {
            Lit::Str(text) => text.value().parse().ok(),
            Lit::ByteStr(bytes) => Some(
                bytes
                    .value()
                    .iter()
                    .find(|byte| char::from(**byte) == Self::LINE_BREAK)
                    .map_or(Pinning::Plain, |_| Pinning::LineBreak),
            ),
            _ => None,
        }
    }

    fn shown(pinning: Pinning, reading: Reading) -> Option<Pinning> {
        match pinning {
            Pinning::LineBreak => Some(Pinning::LineBreak),
            Pinning::Debug => Some(reading).filter(|candidate| *candidate == Reading::Format).map(|_| Pinning::Debug),
            Pinning::Plain => None,
        }
    }

    pub fn called(function: &Expr) -> Option<Name> {
        match function {
            Expr::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        }
    }
}
