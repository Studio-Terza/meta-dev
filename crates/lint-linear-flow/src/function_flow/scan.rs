use std::path::Path;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprMatch, ExprUnary, ExprWhile, Item, Local, Macro};

use lint_core::{Site, SourceFile};
use lint_layout::LineNumber;
use lint_sweep::{Descend, Kind, Watch};

use crate::finding::Finding;
use crate::function_flow::found::Found;
use crate::function_flow::root::Root;

#[derive(Debug)]
pub struct Scan<'file, 'body> {
    path: &'file Path,
    root: Root<'body>,
    found: Vec<Found<'file>>,
}

impl<'file, 'body> Scan<'file, 'body> {
    pub fn new(file: &'file SourceFile, root: Root<'body>) -> Scan<'file, 'body> {
        Scan { path: file.head().path(), root, found: Vec::new() }
    }

    pub fn into_found(self) -> Vec<Found<'file>> {
        self.found
    }

    fn record(&mut self, span: Span, finding: Option<Finding>) -> Descend {
        self.found.extend(finding.map(|flagged| Found::new(Site::new(self.path, LineNumber::from(span)), flagged)));
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Scan<'_, '_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Item,
            Kind::Macro,
            Kind::ExprIf,
            Kind::ExprMatch,
            Kind::ExprWhile,
            Kind::ExprForLoop,
            Kind::ExprLoop,
            Kind::Local,
            Kind::ExprBinary,
            Kind::ExprUnary,
        ]
    }

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        Descend::Over
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_expr_if(&mut self, expr: &'ast ExprIf) -> Descend {
        let finding = self.root.routing_if(expr).map_or_else(|| Some(Finding::branching(&expr.cond)), |_| None);
        self.record(expr.if_token.span, finding)
    }

    fn on_expr_match(&mut self, expr: &'ast ExprMatch) -> Descend {
        let finding = self.root.routing_match(expr).map_or(Some(Finding::Match), |_| None);
        self.record(expr.match_token.span, finding)
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        self.record(expr.while_token.span, Some(Finding::While))
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        self.record(expr.for_token.span, Some(Finding::For))
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        self.record(expr.loop_token.span, Some(Finding::Loop))
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        let otherwise = local.init.as_ref().and_then(|init| init.diverge.as_ref());
        self.found.extend(
            otherwise
                .map(|(token, _)| Found::new(Site::new(self.path, LineNumber::from(token.span)), Finding::LetElse)),
        );
        Descend::Into
    }

    fn on_expr_binary(&mut self, expr: &'ast ExprBinary) -> Descend {
        self.record(expr.op.span(), Finding::joining(expr.op))
    }

    fn on_expr_unary(&mut self, expr: &'ast ExprUnary) -> Descend {
        self.record(expr.op.span(), Finding::negating(expr.op))
    }
}
