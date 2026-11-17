mod root;

use std::path::Path;

use syntax::Span;
use syntax::Spanned;
use syntax::{Block, ExprBinary, ExprIf, ExprMatch, ExprUnary, Local};

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

    #[must_use]
    pub fn entered(self, block: &Block) -> Opened<'file> {
        Opened { body: self.body.filter(|body| *body != Address::from(block)), ..self }
    }

    #[must_use]
    pub fn active(self, change: impl FnOnce(Opened<'file>) -> Opened<'file>) -> Opened<'file> {
        match self.body {
            None => change(self),
            Some(_) => self,
        }
    }

    pub fn closed(self) -> FunctionFlow<'file> {
        FunctionFlow { site: self.site, shape: self.shape, found: self.found.into_iter().chain(self.faults).collect() }
    }

    #[must_use]
    pub fn record(self, span: Span, finding: Option<Finding>) -> Opened<'file> {
        let path = self.path;
        let flagged = finding.map(|flagged| Found::new(Site::new(path, LineNumber::from(span)), flagged));
        Opened { found: self.found.into_iter().chain(flagged).collect(), ..self }
    }

    #[must_use]
    pub fn branched(self, expr: &ExprIf) -> Opened<'file> {
        let rooted = self.root.filter(|root| *root == Address::from(expr));
        self.record(expr.if_token.span, rooted.map_or_else(|| Some(Finding::branching(&expr.cond)), |_| None))
    }

    #[must_use]
    pub fn matched(self, expr: &ExprMatch) -> Opened<'file> {
        let rooted = self.root.filter(|root| *root == Address::from(expr));
        self.record(expr.match_token.span, rooted.map_or(Some(Finding::Match), |_| None))
    }

    #[must_use]
    pub fn bound(self, local: &Local) -> Opened<'file> {
        let otherwise = local.init.as_ref().and_then(|init| init.diverge.as_ref());
        self.record(
            otherwise.map_or_else(Span::call_site, |(token, _)| token.span),
            otherwise.map(|_| Finding::LetElse),
        )
    }

    #[must_use]
    pub fn joined(self, expr: &ExprBinary) -> Opened<'file> {
        self.record(expr.op.span(), Finding::joining(expr.op))
    }

    #[must_use]
    pub fn negated(self, expr: &ExprUnary) -> Opened<'file> {
        self.record(expr.op.span(), Finding::negating(expr.op))
    }
}
