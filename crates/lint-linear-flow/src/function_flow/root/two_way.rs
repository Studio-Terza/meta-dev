use syn::{Block, Expr, ExprBlock, ExprIf, Stmt};

use lint_core::Syntax;

#[derive(Debug)]
pub struct TwoWay;

impl TwoWay {
    pub fn sole(body: &Block) -> Option<&Stmt> {
        body.stmts.split_first().and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
    }

    pub fn within(body: &Block) -> Option<&ExprIf> {
        Self::sole(body)
            .and_then(Self::branching)
            .filter(|routed| Syntax::expr_path(&routed.cond).is_some())
            .filter(|routed| routed.else_branch.as_ref().and_then(|(_, otherwise)| Self::block(otherwise)).is_some())
    }

    fn branching(statement: &Stmt) -> Option<&ExprIf> {
        match statement {
            Stmt::Expr(Expr::If(routed), None) => Some(routed),
            _ => None,
        }
    }

    fn block(otherwise: &Expr) -> Option<&ExprBlock> {
        match otherwise {
            Expr::Block(block) => Some(block),
            _ => None,
        }
    }
}
