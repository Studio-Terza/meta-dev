use syn::visit::{self, Visit};
use syn::{Expr, ExprForLoop, ExprMethodCall, Path};

use lint_core::Macros;
use lint_layout::{Casing, Name};
use newtype::Counter;

use super::{Leaves, Trace};
use crate::check::{Cases, Domain, Rows, Values};

pub(crate) struct Iterations<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    domain: Domain,
}

impl<'trace, 'body, 'file> Iterations<'trace, 'body, 'file> {
    pub(crate) fn within(trace: &'trace Trace<'body, 'file>, expression: &Expr) -> Domain {
        let mut iterations = Iterations { trace, domain: Domain::SINGLE };
        iterations.visit_expr(expression);
        iterations.domain
    }

    pub(crate) fn looped(trace: &'trace Trace<'body, 'file>, looped: &ExprForLoop) -> Domain {
        Iterations { trace, domain: Domain::SINGLE }.root(&looped.expr)
    }

    fn root(&self, expression: &Expr) -> Domain {
        match expression {
            Expr::Array(array) => self.listed(&array.elems.iter().collect::<Vec<&Expr>>()),
            Expr::Reference(reference) => self.root(&reference.expr),
            Expr::Paren(inner) => self.root(&inner.expr),
            Expr::Macro(invocation) => self.listed(&Macros::arguments(&invocation.mac).iter().collect::<Vec<&Expr>>()),
            Expr::Path(path) => {
                self.trace.bound(expression).map_or_else(|| Self::enumerated(&path.path), |bound| self.root(bound))
            }
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

impl<'ast> Visit<'ast> for Iterations<'_, '_, '_> {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.domain = self.domain.max_with(self.root(&call.receiver));
        visit::visit_expr_method_call(self, call);
    }
}
