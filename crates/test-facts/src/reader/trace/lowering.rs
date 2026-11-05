mod composite;
mod operations;

use std::collections::BTreeSet;

use syn::{Expr, Path};

use lint_layout::Name;

use super::{Locals, Trace};
use crate::boundary::Spelling;
use crate::claim::{Callee, Term};
use composite::Composite;
pub(crate) use operations::Operations;

pub(crate) struct Lowering<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    followed: BTreeSet<Name>,
}

impl<'trace, 'body, 'file> Lowering<'trace, 'body, 'file> {
    pub(crate) fn term_of(trace: &'trace Trace<'body, 'file>, expression: &Expr) -> Term {
        Lowering { trace, followed: BTreeSet::new() }.term(expression)
    }

    fn term(&mut self, expression: &Expr) -> Term {
        match expression {
            Expr::Lit(literal) => Operations::literal(&literal.lit),
            Expr::Paren(inner) => self.term(&inner.expr),
            Expr::Group(inner) => self.term(&inner.expr),
            Expr::Reference(inner) => self.term(&inner.expr),
            Expr::Cast(cast) => self.term(&cast.expr),
            Expr::Unary(unary) => Composite::unary(self, unary),
            Expr::Binary(binary) => Composite::binary(self, binary),
            Expr::Path(path) => self.path(expression, &path.path),
            Expr::Call(call) => Composite::call(self, call),
            Expr::MethodCall(call) => Composite::method(self, call),
            Expr::Tuple(tuple) => Callee::Tuple.folded(tuple.elems.iter().map(|element| self.term(element)).collect()),
            Expr::Array(array) => {
                Callee::Sequence.folded(array.elems.iter().map(|element| self.term(element)).collect())
            }
            Expr::Field(field) => Callee::Field(Operations::member(&field.member)).folded(vec![self.term(&field.base)]),
            Expr::Index(index) => Callee::Index.folded(vec![self.term(&index.expr), self.term(&index.index)]),
            Expr::Macro(invocation) => Composite::invoked(self, &invocation.mac, expression),
            _ => Term::Opaque(Spelling::written(expression)),
        }
    }

    fn path(&mut self, expression: &Expr, path: &Path) -> Term {
        let named = path.get_ident().map(Name::from);
        let quantified = named.as_ref().and_then(|name| self.trace.quantifiers().named(name)).cloned();
        quantified.or_else(|| self.followed(expression)).unwrap_or_else(|| Operations::constructor(path, Vec::new()))
    }

    fn followed(&mut self, expression: &Expr) -> Option<Term> {
        let guard = Locals::named(expression);
        let fresh = guard.as_ref().map_or(Some(()), |name| self.followed.insert(name.clone()).then_some(()));
        let found = fresh.and_then(|()| self.trace.bound(expression)).map(|bound| self.term(bound));
        self.followed.retain(|followed| Some(followed) != guard.as_ref());
        found
    }
}
