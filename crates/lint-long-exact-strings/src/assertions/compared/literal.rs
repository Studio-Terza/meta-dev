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
        let pinning = match literal {
            Lit::Str(text) => text.value().parse().ok(),
            Lit::ByteStr(bytes) => Some(if bytes.value().iter().any(|byte| char::from(*byte) == Self::LINE_BREAK) {
                Pinning::LineBreak
            } else {
                Pinning::Plain
            }),
            _ => None,
        };
        pinning.filter(|pinning| match (reading, pinning) {
            (_, Pinning::LineBreak) | (Reading::Format, Pinning::Debug) => true,
            (Reading::Plain, Pinning::Debug) | (_, Pinning::Plain) => false,
        })
    }

    pub fn called(function: &Expr) -> Option<Name> {
        match function {
            Expr::Path(path) => path.path.segments.last().map(|last| Name::from(&last.ident)),
            _ => None,
        }
    }
}
