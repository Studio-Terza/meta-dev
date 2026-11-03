mod arms;

use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprIf, Stmt};

use crate::function_flow::found::Found;
use crate::function_flow::scan::Scan;
use crate::verdict::Verdict;
use arms::Arms;

#[derive(Clone, Copy, Debug)]
pub enum Root<'ast> {
    Match(&'ast syn::ExprMatch),
    IfLet(&'ast ExprIf),
    Body(&'ast Block),
}

impl<'ast> From<&'ast Block> for Root<'ast> {
    fn from(body: &'ast Block) -> Root<'ast> {
        body.stmts
            .split_first()
            .and_then(|(only, rest)| rest.first().map_or(Some(only), |_| None))
            .map_or(Root::Body(body), |only| Self::of_statement(only, body))
    }
}

impl<'ast> Root<'ast> {
    fn of_statement(statement: &'ast Stmt, body: &'ast Block) -> Root<'ast> {
        match statement {
            Stmt::Expr(Expr::Match(routed), None) => Root::Match(routed),
            Stmt::Expr(Expr::If(routed), None) => Self::of_if(routed, body),
            _ => Root::Body(body),
        }
    }

    fn of_if(routed: &'ast ExprIf, body: &'ast Block) -> Root<'ast> {
        match &*routed.cond {
            Expr::Let(_) => Root::IfLet(routed),
            _ => Root::Body(body),
        }
    }

    pub fn verdict(self) -> Verdict {
        match self {
            Root::Match(_) | Root::IfLet(_) => Verdict::Router,
            Root::Body(_) => Verdict::Pipeline,
        }
    }

    pub fn scanned_by(self, scan: &mut Scan<'_>) {
        match self {
            Root::Match(routed) => visit::visit_expr_match(scan, routed),
            Root::IfLet(routed) => visit::visit_expr_if(scan, routed),
            Root::Body(body) => scan.visit_block(body),
        }
    }

    pub fn faults(self, path: &Path) -> Vec<Found<'_>> {
        match self {
            Root::Match(routed) => Arms::new(path).matched(routed),
            Root::IfLet(routed) => Arms::new(path).branched(routed),
            Root::Body(_) => Vec::new(),
        }
    }
}
