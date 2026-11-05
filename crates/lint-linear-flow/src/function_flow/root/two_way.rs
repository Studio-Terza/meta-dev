use std::cmp::Ordering;

use syn::{Block, Expr, ExprBlock, ExprIf, Ident, Path, Stmt};

use lint_core::Syntax;

#[derive(Debug)]
pub struct TwoWay;

impl TwoWay {
    pub fn sole(body: &Block) -> Option<&Stmt> {
        body.stmts.split_first().and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
    }

    pub fn within<'body>(body: &'body Block, input: &Ident) -> Option<&'body ExprIf> {
        Self::sole(body)
            .and_then(Self::branching)
            .filter(|routed| Syntax::expr_path(&routed.cond).is_some_and(|cond| cond.path.is_ident(input)))
            .filter(|routed| Variants::between(routed).is_some())
    }

    fn branching(statement: &Stmt) -> Option<&ExprIf> {
        match statement {
            Stmt::Expr(Expr::If(routed), None) => Some(routed),
            _ => None,
        }
    }

    pub fn block(otherwise: &Expr) -> Option<&ExprBlock> {
        match otherwise {
            Expr::Block(block) => Some(block),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Variants;

impl Variants {
    pub fn between(routed: &ExprIf) -> Option<Ordering> {
        let (_, otherwise) = routed.else_branch.as_ref()?;
        let chosen = Self::unit(&routed.then_branch)?;
        let other = Self::unit(&TwoWay::block(otherwise)?.block)?;
        Some(Self::qualifier(chosen).cmp(Self::qualifier(other)))
            .filter(|order| order.is_eq())
            .filter(|_| chosen.segments.len().cmp(&1).is_gt())
    }

    fn qualifier(path: &Path) -> impl Iterator<Item = &Ident> {
        path.segments.iter().rev().skip(1).map(|segment| &segment.ident)
    }

    fn unit(branch: &Block) -> Option<&Path> {
        TwoWay::sole(branch).and_then(Self::tail).and_then(Syntax::expr_path).map(|bare| &bare.path)
    }

    fn tail(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(expr, None) => Some(expr),
            _ => None,
        }
    }
}
