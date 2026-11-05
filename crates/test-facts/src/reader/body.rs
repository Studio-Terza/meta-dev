use std::collections::BTreeSet;
use std::mem;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprCall, ExprForLoop, ExprMethodCall, Local, Macro};

use lint_core::{MacroAliases, Syntax};
use lint_layout::Name;

use super::assertion::Assertion;
use super::scope::Scope;
use super::trace::{Iterations, Locals, Trace};
use crate::check::{Check, Domain};

pub(crate) struct Body<'scope, 'file> {
    scope: &'scope Scope<'file>,
    tables: &'scope MacroAliases,
    locals: Locals<'file>,
    domain: Domain,
    open: BTreeSet<Name>,
    checks: Vec<Check>,
}

impl<'scope, 'file> Body<'scope, 'file> {
    pub(crate) fn checks(
        scope: &'scope Scope<'file>,
        tables: &'scope MacroAliases,
        block: &'file Block,
        domain: Domain,
    ) -> Vec<Check> {
        let mut body =
            Body { scope, tables, locals: Locals::default(), domain, open: BTreeSet::new(), checks: Vec::new() };
        body.visit_block(block);
        body.checks
    }

    fn met(&mut self, invocation: &Macro) {
        let assertion = Assertion::named(invocation, self.tables);
        let found = assertion.checks(invocation, &Trace::new(self.scope, &self.locals), self.domain);
        self.checks.extend(found);
        assertion.nested(invocation).iter().for_each(|inner| self.met(inner));
    }

    fn called(&mut self, callee: Option<Name>, arguments: &[&'file Expr]) {
        self.fresh(callee).iter().for_each(|name| self.entered(name, arguments));
    }

    fn fresh(&self, callee: Option<Name>) -> Vec<Name> {
        callee.into_iter().collect::<BTreeSet<Name>>().difference(&self.open).cloned().collect()
    }

    fn entered(&mut self, name: &Name, arguments: &[&'file Expr]) {
        self.open.insert(name.clone());
        let scope = self.scope;
        scope.helpers(name).iter().for_each(|helper| {
            let inner = helper.called(arguments, &self.locals);
            let outer = mem::replace(&mut self.locals, inner);
            helper.visited(self);
            self.locals = outer;
        });
        self.open.remove(name);
    }
}

impl<'file> Visit<'file> for Body<'_, 'file> {
    fn visit_local(&mut self, local: &'file Local) {
        self.locals.bind(local);
        visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, looped: &'file ExprForLoop) {
        let outer = self.domain;
        self.domain = outer.max_with(Iterations::looped(&Trace::new(self.scope, &self.locals), looped));
        visit::visit_expr_for_loop(self, looped);
        self.domain = outer;
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }

    fn visit_expr_call(&mut self, call: &'file ExprCall) {
        self.called(
            Syntax::expr_path(&call.func)
                .and_then(|path| path.path.segments.last())
                .map(|last| Name::from(&last.ident)),
            &call.args.iter().collect::<Vec<&Expr>>(),
        );
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'file ExprMethodCall) {
        self.called(Some(Name::from(&call.method)), &call.args.iter().collect::<Vec<&Expr>>());
        visit::visit_expr_method_call(self, call);
    }
}
