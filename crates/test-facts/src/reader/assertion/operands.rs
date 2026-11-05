use syn::{Expr, Macro};

use lint_core::Macros;

use super::super::trace::{Lowering, Trace};
use super::condition::Condition;
use crate::claim::{Accepted, Claim, Exclusion};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Comparing {
    Equal,
    Unequal,
}

pub(crate) struct Operands {
    left: Expr,
    right: Expr,
}

impl Operands {
    pub(crate) fn read(invocation: &Macro) -> Option<Operands> {
        let arguments = Macros::arguments(invocation);
        arguments
            .first()
            .zip(arguments.get(1))
            .map(|(left, right)| Operands { left: left.clone(), right: right.clone() })
    }

    pub(crate) fn claims(&self, trace: &Trace<'_, '_>, comparing: Comparing) -> Vec<Claim> {
        match comparing {
            Comparing::Equal => Lowering::term_of(trace, &self.right)
                .truth()
                .map_or_else(|| self.equality(trace), |truth| Condition::held(trace, &self.left, truth)),
            Comparing::Unequal => {
                trace.stated(&self.left, Accepted::Excluding(Exclusion::from(Lowering::term_of(trace, &self.right))))
            }
        }
    }

    fn equality(&self, trace: &Trace<'_, '_>) -> Vec<Claim> {
        Lowering::term_of(trace, &self.left).truth().map_or_else(
            || trace.stated(&self.left, Accepted::equal(Lowering::term_of(trace, &self.right))),
            |truth| Condition::held(trace, &self.right, truth),
        )
    }
}
