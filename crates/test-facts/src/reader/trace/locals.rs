use std::collections::BTreeMap;

use syn::{Expr, Local};

use lint_core::Syntax;
use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Locals<'file> {
    bound: BTreeMap<Name, &'file Expr>,
}

impl<'file> Locals<'file> {
    pub(crate) fn bind(&mut self, local: &'file Local) {
        let bound = Syntax::pat_ident(&local.pat).zip(local.init.as_ref());
        self.bound.extend(bound.map(|(name, init)| (Name::from(&name.ident), &*init.expr)));
    }

    pub(crate) fn binding(mut self, name: Name, value: &'file Expr) -> Locals<'file> {
        self.bound.insert(name, value);
        self
    }

    pub(crate) fn resolved(&self, argument: &'file Expr) -> &'file Expr {
        Locals::named(argument).and_then(|name| self.bound(&name)).unwrap_or(argument)
    }

    pub(crate) fn bound(&self, name: &Name) -> Option<&'file Expr> {
        self.bound.get(name).copied()
    }

    pub(crate) fn named(expression: &Expr) -> Option<Name> {
        Syntax::expr_path(expression).and_then(|path| path.path.get_ident()).map(Name::from)
    }
}
