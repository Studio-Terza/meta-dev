use std::collections::{BTreeMap, BTreeSet};

use syn::{Expr, Local, LocalInit, PatIdent};

use lint_core::Syntax;
use lint_layout::Name;

use crate::boundary::Spelling;
use crate::claim::Term;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Binding<'file> {
    Written(&'file Expr),
    State(Term),
}

impl<'file> Binding<'file> {
    fn written(&self) -> Option<&'file Expr> {
        match self {
            Binding::Written(expression) => Some(expression),
            Binding::State(_) => None,
        }
    }

    fn state(&self) -> Option<&Term> {
        match self {
            Binding::State(term) => Some(term),
            Binding::Written(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Locals<'file> {
    bound: BTreeMap<Name, Binding<'file>>,
    mutable: BTreeSet<Name>,
}

impl<'file> Locals<'file> {
    pub(crate) fn bind(&mut self, local: &'file Local) {
        let bound = Syntax::pat_ident(&local.pat).zip(local.init.as_ref());
        bound.map(|(name, init)| self.bound_as(name, init)).unwrap_or_default();
    }

    pub(crate) fn bound_as(&mut self, name: &PatIdent, init: &'file LocalInit) {
        let own = Name::from(&name.ident);
        self.bound.insert(own.clone(), Binding::Written(&init.expr));
        self.mutable.remove(&own);
        self.mutable.extend(name.mutability.map(|_| own));
    }

    pub(crate) fn binding(mut self, name: Name, value: &'file Expr) -> Locals<'file> {
        self.bound.insert(name, Binding::Written(value));
        self
    }

    pub(crate) fn resolved(&self, argument: &'file Expr) -> &'file Expr {
        Locals::named(argument).and_then(|name| self.bound(&name)).unwrap_or(argument)
    }

    pub(crate) fn bound(&self, name: &Name) -> Option<&'file Expr> {
        self.bound.get(name).and_then(Binding::written)
    }

    pub(crate) fn state(&self, name: &Name) -> Option<&Term> {
        self.bound.get(name).and_then(Binding::state)
    }

    pub(crate) fn mutable(&self, name: &Name) -> Option<&Name> {
        self.mutable.get(name)
    }

    pub(crate) fn restated(&self, name: Name, term: Term) -> Locals<'file> {
        let mut next = self.clone();
        next.bound.insert(name, Binding::State(term));
        next
    }

    pub(crate) fn unsettled(&self, before: &Locals<'file>) -> Locals<'file> {
        let mut next = self.clone();
        self.bound.iter().filter(|(name, binding)| before.bound.get(*name) != Some(*binding)).for_each(|(name, _)| {
            next.bound.insert(name.clone(), Binding::State(Term::Opaque(Spelling::from(name))));
        });
        next
    }

    pub(crate) fn rooted(expression: &Expr) -> Option<&Expr> {
        match expression {
            Expr::Path(_) => Some(expression),
            Expr::Field(field) => Locals::rooted(&field.base),
            Expr::Index(index) => Locals::rooted(&index.expr),
            Expr::Paren(inner) => Locals::rooted(&inner.expr),
            Expr::Reference(inner) => Locals::rooted(&inner.expr),
            Expr::Unary(inner) => Locals::rooted(&inner.expr),
            Expr::MethodCall(call) => Locals::rooted(&call.receiver),
            _ => None,
        }
    }

    pub(crate) fn named(expression: &Expr) -> Option<Name> {
        Syntax::expr_path(expression).and_then(|path| path.path.get_ident()).map(Name::from)
    }
}
