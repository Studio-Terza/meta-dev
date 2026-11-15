mod arms;

use std::path::Path;

use syn::{Block, Expr, ExprIf, ExprMatch, Stmt};

use crate::function_flow::found::Found;
use crate::verdict::Verdict;
use arms::Arms;
use lint_sweep::Address;

#[derive(Clone, Copy, Debug)]
pub enum Root<'ast> {
    Match(&'ast ExprMatch),
    IfLet(&'ast ExprIf),
    Body,
}

impl<'ast> From<&'ast Block> for Root<'ast> {
    fn from(body: &'ast Block) -> Root<'ast> {
        body.stmts
            .split_first()
            .and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
            .map_or(Root::Body, Self::of_statement)
    }
}

impl<'ast> Root<'ast> {
    fn of_statement(statement: &'ast Stmt) -> Root<'ast> {
        match statement {
            Stmt::Expr(Expr::Match(routed), None) => Root::Match(routed),
            Stmt::Expr(Expr::If(routed), None) => Self::of_if(routed),
            _ => Root::Body,
        }
    }

    fn of_if(routed: &'ast ExprIf) -> Root<'ast> {
        match &*routed.cond {
            Expr::Let(_) => Root::IfLet(routed),
            _ => Root::Body,
        }
    }

    pub fn verdict(self) -> Verdict {
        match self {
            Root::Match(_) | Root::IfLet(_) => Verdict::Router,
            Root::Body => Verdict::Pipeline,
        }
    }

    pub fn address(self) -> Option<Address> {
        match self {
            Root::Match(routed) => Some(Address::from(routed)),
            Root::IfLet(routed) => Some(Address::from(routed)),
            Root::Body => None,
        }
    }

    pub fn faults(self, path: &Path) -> Vec<Found<'_>> {
        match self {
            Root::Match(routed) => Arms::new(path).matched(routed),
            Root::IfLet(routed) => Arms::new(path).branched(routed),
            Root::Body => Vec::new(),
        }
    }
}
