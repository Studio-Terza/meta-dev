use syn::{Block, Expr, Stmt};

use crate::finding::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmShape {
    Single,
    Compound,
}

impl From<&Expr> for ArmShape {
    fn from(arm: &Expr) -> Self {
        match arm {
            Expr::Call(_)
            | Expr::MethodCall(_)
            | Expr::Path(_)
            | Expr::Lit(_)
            | Expr::Struct(_)
            | Expr::Tuple(_)
            | Expr::Array(_)
            | Expr::Repeat(_)
            | Expr::Macro(_)
            | Expr::Reference(_)
            | Expr::Field(_)
            | Expr::Index(_)
            | Expr::Cast(_)
            | Expr::Closure(_)
            | Expr::Unary(_)
            | Expr::Range(_)
            | Expr::Try(_)
            | Expr::Await(_) => Self::Single,
            Expr::Paren(inner) => Self::from(&*inner.expr),
            Expr::Block(inner) => Self::from(&inner.block),
            _ => Self::Compound,
        }
    }
}

impl From<&Block> for ArmShape {
    fn from(block: &Block) -> Self {
        block
            .stmts
            .split_first()
            .map_or(Self::Single, |(first, rest)| rest.first().map_or_else(|| Self::from(first), |_| Self::Compound))
    }
}

impl From<&Stmt> for ArmShape {
    fn from(statement: &Stmt) -> Self {
        match statement {
            Stmt::Expr(expr, _) => Self::from(expr),
            Stmt::Macro(_) => Self::Single,
            Stmt::Local(_) | Stmt::Item(_) => Self::Compound,
        }
    }
}

impl ArmShape {
    pub fn fault(self) -> Option<Finding> {
        match self {
            Self::Single => None,
            Self::Compound => Some(Finding::ArmLogic),
        }
    }
}
