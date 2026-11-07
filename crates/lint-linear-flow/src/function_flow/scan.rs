use std::path::Path;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, Item, Local, Macro};

use lint_core::{Macros, Site};
use lint_layout::LineNumber;

use crate::finding::Finding;
use crate::function_flow::found::Found;

pub struct Scan<'file> {
    path: &'file Path,
    found: Vec<Found<'file>>,
}

impl<'file> Scan<'file> {
    pub const fn new(path: &'file Path) -> Self {
        Scan { path, found: Vec::new() }
    }

    pub fn into_found(self) -> Vec<Found<'file>> {
        self.found
    }

    fn record(&mut self, span: Span, finding: Option<Finding>) {
        self.found.extend(finding.map(|finding| Found::new(Site::new(self.path, LineNumber::from(span)), finding)));
    }
}

impl<'ast> Visit<'ast> for Scan<'_> {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.record(expr.if_token.span, Some(Finding::branching(&expr.cond)));
        visit::visit_expr_if(self, expr);
    }

    fn visit_expr_match(&mut self, expr: &'ast ExprMatch) {
        self.record(expr.match_token.span, Some(Finding::Match));
        visit::visit_expr_match(self, expr);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.record(expr.while_token.span, Some(Finding::While));
        visit::visit_expr_while(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.record(expr.for_token.span, Some(Finding::For));
        visit::visit_expr_for_loop(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.record(expr.loop_token.span, Some(Finding::Loop));
        visit::visit_expr_loop(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        local
            .init
            .iter()
            .filter_map(|init| init.diverge.as_ref())
            .for_each(|(otherwise, _)| self.record(otherwise.span, Some(Finding::LetElse)));
        visit::visit_local(self, local);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        self.record(expr.op.span(), Finding::joining(expr.op));
        visit::visit_expr_binary(self, expr);
    }

    fn visit_expr_unary(&mut self, expr: &'ast ExprUnary) {
        self.record(expr.op.span(), Finding::negating(expr.op));
        visit::visit_expr_unary(self, expr);
    }
}
