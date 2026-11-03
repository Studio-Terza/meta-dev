use syn::{Block, Expr, Stmt};

use crate::finding::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmShape {
    Single,
    Compound,
}

impl From<&Expr> for ArmShape {
    fn from(arm: &Expr) -> ArmShape {
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
            | Expr::Await(_) => ArmShape::Single,
            Expr::Paren(inner) => ArmShape::from(&*inner.expr),
            Expr::Block(inner) => ArmShape::from(&inner.block),
            _ => ArmShape::Compound,
        }
    }
}

impl From<&Block> for ArmShape {
    fn from(block: &Block) -> ArmShape {
        block.stmts.split_first().map_or(ArmShape::Single, |(first, rest)| {
            rest.first().map_or_else(|| ArmShape::from(first), |_| ArmShape::Compound)
        })
    }
}

impl From<&Stmt> for ArmShape {
    fn from(statement: &Stmt) -> ArmShape {
        match statement {
            Stmt::Expr(expr, _) => ArmShape::from(expr),
            Stmt::Macro(_) => ArmShape::Single,
            Stmt::Local(_) | Stmt::Item(_) => ArmShape::Compound,
        }
    }
}

impl ArmShape {
    pub fn fault(self) -> Option<Finding> {
        match self {
            ArmShape::Single => None,
            ArmShape::Compound => Some(Finding::ArmLogic),
        }
    }
}
