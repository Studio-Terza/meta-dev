use syn::{Expr, ExprForLoop, Path};

use lint_core::Macros;
use lint_layout::{Casing, Name, Whole, Word, words};
use newtype::Counter;

use super::{Constants, Leaves, Trace};
use crate::check::{Cases, Domain, Rows, Values};

pub(crate) struct Iterations<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
}

impl<'trace, 'body, 'file> Iterations<'trace, 'body, 'file> {
    const ADAPTERS: [Word<'static>; 6] = words!["iter", "into_iter", "copied", "cloned", "enumerate", "rev"];

    pub(crate) fn looped(trace: &'trace Trace<'body, 'file>, looped: &ExprForLoop) -> Domain {
        Iterations { trace }.root(&looped.expr)
    }

    fn root(&self, expression: &Expr) -> Domain {
        match expression {
            Expr::Array(array) => self.listed(&array.elems.iter().collect::<Vec<&Expr>>()),
            Expr::Reference(reference) => self.root(&reference.expr),
            Expr::Paren(inner) => self.root(&inner.expr),
            Expr::Macro(invocation) => self.listed(&Macros::arguments(&invocation.mac).iter().collect::<Vec<&Expr>>()),
            Expr::Path(path) => self
                .trace
                .bound(expression, Constants::Followed)
                .map_or_else(|| Self::enumerated(&path.path), |bound| self.root(bound)),
            Expr::MethodCall(call) => Name::from(&call.method)
                .matches::<Whole>(&Self::ADAPTERS)
                .map_or(Domain::SINGLE, |_| self.root(&call.receiver)),
            _ => Domain::SINGLE,
        }
    }

    fn listed(&self, elements: &[&Expr]) -> Domain {
        let leaves = Values::total(elements.iter().map(|element| Leaves::counted(self.trace, element)));
        Domain::listed(Rows::new(Cases::new(elements.len()), leaves))
    }

    fn enumerated(path: &Path) -> Domain {
        let names: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        names
            .iter()
            .rev()
            .nth(1)
            .filter(|owner| owner.casing() == Casing::Camel)
            .and(names.last())
            .filter(|member| member.casing() == Casing::Shouting)
            .map_or(Domain::SINGLE, |_| Domain::ENUMERATED)
    }
}
