mod arm_shape;
mod dispatch;

use std::path::Path;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Arm, Expr, ExprIf, ExprMatch};

use lint_core::Site;
use lint_layout::LineNumber;

use crate::finding::Finding;
use crate::function_flow::found::Found;
use arm_shape::ArmShape;
use dispatch::Dispatch;

#[derive(Clone, Copy, Debug)]
pub struct Arms<'file> {
    path: &'file Path,
}

impl<'file> Arms<'file> {
    pub fn new(path: &'file Path) -> Self {
        Arms { path }
    }

    pub fn matched(self, routed: &ExprMatch) -> Vec<Found<'file>> {
        routed.arms.iter().flat_map(|arm| self.arm_faults(arm)).collect()
    }

    pub fn branched(self, routed: &ExprIf) -> Vec<Found<'file>> {
        self.condition_fault(&routed.cond)
            .into_iter()
            .chain(self.located(routed.then_branch.span(), ArmShape::from(&routed.then_branch).fault()))
            .chain(
                routed
                    .else_branch
                    .as_ref()
                    .and_then(|(_, otherwise)| self.located(otherwise.span(), ArmShape::from(&**otherwise).fault())),
            )
            .collect()
    }

    fn located(self, span: Span, finding: Option<Finding>) -> Option<Found<'file>> {
        finding.map(|finding| Found::new(Site::new(self.path, LineNumber::from(span)), finding))
    }

    fn arm_faults(self, arm: &Arm) -> Vec<Found<'file>> {
        arm.guard
            .iter()
            .filter_map(|(guard, _)| self.located(guard.span, Some(Finding::Guard)))
            .chain(self.located(arm.pat.span(), Dispatch::from(&arm.pat).fault()))
            .chain(self.located(arm.body.span(), ArmShape::from(&*arm.body).fault()))
            .collect()
    }

    fn condition_fault(self, condition: &Expr) -> Option<Found<'file>> {
        match condition {
            Expr::Let(binding) => self.located(binding.pat.span(), Dispatch::from(&*binding.pat).fault()),
            _ => None,
        }
    }
}
