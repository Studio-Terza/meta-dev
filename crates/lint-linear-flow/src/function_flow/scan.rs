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
use root::Root;

#[derive(Clone, Copy, Debug)]
pub struct Candidate<'file> {
    found: Found<'file>,
    at: Option<Address>,
    span: Span,
}

impl<'file> Candidate<'file> {
    fn recorded(
        path: &'file Path,
        span: Span,
        finding: Option<Finding>,
        at: Option<Address>,
    ) -> Option<Candidate<'file>> {
        finding.map(|flagged| Candidate {
            found: Found::new(Site::new(path, LineNumber::from(span)), flagged),
            at,
            span,
        })
    }

    pub fn looped(path: &'file Path, span: Span, finding: Finding) -> Option<Candidate<'file>> {
        Self::recorded(path, span, Some(finding), None)
    }

    pub fn branched(path: &'file Path, expr: &ExprIf) -> Option<Candidate<'file>> {
        Self::recorded(path, expr.if_token.span, Some(Finding::branching(&expr.cond)), Some(Address::from(expr)))
    }

    pub fn matched(path: &'file Path, expr: &ExprMatch) -> Option<Candidate<'file>> {
        Self::recorded(path, expr.match_token.span, Some(Finding::Match), Some(Address::from(expr)))
    }

    pub fn bound(path: &'file Path, local: &Local) -> Option<Candidate<'file>> {
        let otherwise = local.init.as_ref().and_then(|init| init.diverge.as_ref());
        Self::recorded(
            path,
            otherwise.map_or_else(Span::call_site, |(token, _)| token.span),
            otherwise.map(|_| Finding::LetElse),
            None,
        )
    }

    pub fn joined(path: &'file Path, expr: &ExprBinary) -> Option<Candidate<'file>> {
        Self::recorded(path, expr.op.span(), Finding::joining(expr.op), None)
    }

    pub fn negated(path: &'file Path, expr: &ExprUnary) -> Option<Candidate<'file>> {
        Self::recorded(path, expr.op.span(), Finding::negating(expr.op), None)
    }
}

impl<'file> FunctionFlow<'file> {
    pub fn closed(
        path: &'file Path,
        name: Span,
        body: &Block,
        candidates: Vec<Candidate<'file>>,
    ) -> FunctionFlow<'file> {
        let root = Root::from(body);
        let rooted = root.address();
        let inside = body.span().byte_range();
        let found = candidates
            .into_iter()
            .filter(|candidate| inside.contains(&candidate.span.byte_range().start))
            .filter(|candidate| candidate.at.is_none_or(|at| Some(at) != rooted));
        FunctionFlow {
            site: Site::new(path, LineNumber::from(name)),
            shape: root.verdict(),
            found: found.map(|candidate| candidate.found).chain(root.faults(path)).collect(),
        }
    }
}
