use std::collections::BTreeSet;
use std::mem;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprCall, ExprForLoop, Local, Macro, Path};

use lint_core::{MacroAliases, Syntax};
use lint_layout::Name;

use super::assertion::Assertion;
use super::scope::{Helper, Scope, Target};
use super::trace::{Iterations, Locals, Trace};
use crate::check::{Check, Domain};

pub(crate) struct Body<'scope, 'file> {
    scope: &'scope Scope<'file>,
    tables: &'scope MacroAliases,
    locals: Locals<'file>,
    domain: Domain,
    open: BTreeSet<Target>,
    owner: Option<Name>,
    checks: Vec<Check>,
}

impl<'scope, 'file> Body<'scope, 'file> {
    pub(crate) fn checks(
        scope: &'scope Scope<'file>,
        tables: &'scope MacroAliases,
        block: &'file Block,
        domain: Domain,
    ) -> Vec<Check> {
        let mut body = Body {
            scope,
            tables,
            locals: Locals::default(),
            domain,
            open: BTreeSet::new(),
            owner: None,
            checks: Vec::new(),
        };
        body.visit_block(block);
        body.checks
    }

    fn met(&mut self, invocation: &Macro) {
        let assertion = Assertion::named(invocation, self.tables);
        let found = assertion.checks(invocation, &Trace::new(self.scope, &self.locals), self.domain);
        self.checks.extend(found);
        assertion.nested(invocation).iter().for_each(|inner| self.met(inner));
    }

    fn called(&mut self, callee: Option<&Path>, arguments: &[&'file Expr]) {
        let scope = self.scope;
        let owner = self.owner.clone();
        callee
            .map(|path| scope.helpers(path, owner.as_ref(), arguments))
            .unwrap_or_default()
            .iter()
            .for_each(|helper| self.entered(helper, arguments));
    }

    fn entered(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr]) {
        let key = helper.target().clone();
        self.open.insert(key.clone()).then(|| self.expanded(helper, arguments, &key));
    }

    fn expanded(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr], key: &Target) {
        let inner = helper.called(arguments, &self.locals);
        let outer = mem::replace(&mut self.locals, inner);
        let owner = mem::replace(&mut self.owner, helper.owner());
        helper.visited(self);
        self.owner = owner;
        self.locals = outer;
        self.open.remove(key);
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
        self.called(Syntax::expr_path(&call.func).map(|path| &path.path), &call.args.iter().collect::<Vec<&Expr>>());
        visit::visit_expr_call(self, call);
    }
}
