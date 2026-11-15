mod root;

use std::path::Path;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Block, ExprBinary, ExprIf, ExprMatch, ExprUnary, Local};

use lint_core::Site;
use lint_layout::LineNumber;
use lint_sweep::Address;

use super::FunctionFlow;
use crate::finding::Finding;
use crate::function_flow::found::Found;
use crate::verdict::Verdict;
use root::Root;

#[derive(Debug)]
pub struct Opened<'file> {
    path: &'file Path,
    body: Option<Address>,
    site: Site<'file>,
    shape: Verdict,
    root: Option<Address>,
    found: Vec<Found<'file>>,
    faults: Vec<Found<'file>>,
}

impl<'file> Opened<'file> {
    pub fn new(path: &'file Path, name: Span, body: &Block) -> Opened<'file> {
        let root = Root::from(body);
        Opened {
            path,
            body: Some(Address::from(body)),
            site: Site::new(path, LineNumber::from(name)),
            shape: root.verdict(),
            root: root.address(),
            found: Vec::new(),
            faults: root.faults(path),
        }
    }

    pub fn entered(&mut self, block: &Block) {
        self.body = self.body.filter(|body| *body != Address::from(block));
    }

    pub fn active(&mut self) -> Option<&mut Opened<'file>> {
        self.body.map_or(Some(self), |_| None)
    }

    pub fn closed(self) -> FunctionFlow<'file> {
        FunctionFlow { site: self.site, shape: self.shape, found: self.found.into_iter().chain(self.faults).collect() }
    }

    pub fn record(&mut self, span: Span, finding: Option<Finding>) {
        let path = self.path;
        self.found.extend(finding.map(|flagged| Found::new(Site::new(path, LineNumber::from(span)), flagged)));
    }

    pub fn branched(&mut self, expr: &ExprIf) {
        let rooted = self.root.filter(|root| *root == Address::from(expr));
        self.record(expr.if_token.span, rooted.map_or_else(|| Some(Finding::branching(&expr.cond)), |_| None));
    }

    pub fn matched(&mut self, expr: &ExprMatch) {
        let rooted = self.root.filter(|root| *root == Address::from(expr));
        self.record(expr.match_token.span, rooted.map_or(Some(Finding::Match), |_| None));
    }

    pub fn bound(&mut self, local: &Local) {
        let otherwise = local.init.as_ref().and_then(|init| init.diverge.as_ref());
        self.record(
            otherwise.map_or_else(Span::call_site, |(token, _)| token.span),
            otherwise.map(|_| Finding::LetElse),
        );
    }

    pub fn joined(&mut self, expr: &ExprBinary) {
        self.record(expr.op.span(), Finding::joining(expr.op));
    }

    pub fn negated(&mut self, expr: &ExprUnary) {
        self.record(expr.op.span(), Finding::negating(expr.op));
    }
}
