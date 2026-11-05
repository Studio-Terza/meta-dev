use syn::{Expr, ExprForLoop};

use lint_core::Macros;
use lint_layout::{Name, Whole, Word, words};

use super::{Extent, Lowering, Trace};
use crate::claim::{Binder, Domain, Draw, Listed};

pub(crate) struct Iterations<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
}

impl<'trace, 'body, 'file> Iterations<'trace, 'body, 'file> {
    const ADAPTERS: [Word<'static>; 5] = words!["iter", "into_iter", "copied", "cloned", "rev"];

    pub(crate) fn looped(trace: &'trace Trace<'body, 'file>, looped: &ExprForLoop) -> Binder {
        Binder::new(Iterations { trace }.root(&looped.expr), Draw::Every)
    }

    fn root(&self, expression: &Expr) -> Domain {
        match expression {
            Expr::Array(array) => Domain::Listed(self.listed(array.elems.iter())),
            Expr::Reference(reference) => self.root(&reference.expr),
            Expr::Paren(inner) => self.root(&inner.expr),
            Expr::Range(range) => Extent::ranged(self.trace, range)
                .map_or_else(|| Domain::Drawn(Lowering::term_of(self.trace, expression)), Domain::Ranged),
            Expr::Macro(invocation) => Domain::Listed(self.listed(Macros::arguments(&invocation.mac).iter())),
            Expr::Path(_) => self
                .trace
                .bound(expression)
                .map_or_else(|| Domain::Drawn(Lowering::term_of(self.trace, expression)), |bound| self.root(bound)),
            Expr::MethodCall(call) => Name::from(&call.method).matches::<Whole>(&Self::ADAPTERS).map_or_else(
                || Domain::Drawn(Lowering::term_of(self.trace, expression)),
                |_| self.root(&call.receiver),
            ),
            _ => Domain::Drawn(Lowering::term_of(self.trace, expression)),
        }
    }

    fn listed<'element>(&self, elements: impl Iterator<Item = &'element Expr>) -> Listed {
        Listed::from_iter(elements.map(|element| Lowering::term_of(self.trace, element)))
    }
}
