use std::collections::BTreeSet;
use std::mem;

use newtype::Counter;
use syn::visit::{self, Visit};
use syn::{Expr, ExprPath, Lit, Macro};

use lint_core::Macros;
use lint_layout::{Casing, Name};

use super::{Constants, Locals, Trace};
use crate::check::Values;

pub(crate) struct Leaves<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
    followed: BTreeSet<Name>,
    count: Values,
    constants: Constants,
}

impl<'trace, 'body, 'file> Leaves<'trace, 'body, 'file> {
    pub(crate) fn counted(trace: &'trace Trace<'body, 'file>, expression: &Expr) -> Values {
        let mut leaves = Leaves {
            trace,
            followed: BTreeSet::new(),
            count: Values::zero(),
            constants: Constants::followed_in(expression),
        };
        leaves.visit_expr(expression);
        leaves.count
    }

    fn named(&mut self, path: &ExprPath) {
        let last = path.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        let named = Some(last.casing()).filter(|casing| matches!(casing, Casing::Camel | Casing::Shouting));
        self.count = named.map_or(self.count, |_| self.count.incremented());
    }

    fn entered(&mut self, bound: &'file Expr) {
        let outer = mem::replace(&mut self.constants, Constants::followed_in(bound));
        self.visit_expr(bound);
        self.constants = outer;
    }

    fn followed(&mut self, fresh: Option<&'file Expr>, path: &ExprPath) {
        match fresh {
            Some(bound) => self.entered(bound),
            None => self.named(path),
        }
    }
}

impl<'ast> Visit<'ast> for Leaves<'_, '_, '_> {
    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_lit(&mut self, literal: &'ast Lit) {
        self.count = self.count.incremented();
        visit::visit_lit(self, literal);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        let expression = Expr::Path(path.clone());
        let name = Locals::named(&expression).filter(|name| self.followed.insert(name.clone()));
        let fresh = name.and_then(|_| self.trace.bound(&expression, self.constants));
        self.followed(fresh, path);
    }
}
