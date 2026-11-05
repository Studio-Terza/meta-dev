mod reach;

use std::collections::BTreeSet;
use std::mem;

use syn::visit::{self, Visit};
use syn::{Block, Expr, ExprCall, ExprForLoop, ExprIf, ExprWhile, Local, Macro, Path};

use lint_core::Syntax;

use super::assertion::Assertion;
use super::scope::{Helper, Place, Route, Scope};
use super::trace::{Evaluation, Iterations, Locals, Quantifiers, Trace};
use crate::boundary::Truth;
use crate::claim::Claim;
use reach::{Flow, Reach};

pub(crate) struct Body<'scope, 'file> {
    scope: &'scope Scope<'file>,
    locals: Locals<'file>,
    quantifiers: Quantifiers,
    open: BTreeSet<Route>,
    place: Place,
    claims: Vec<Claim>,
}

impl<'scope, 'file> Body<'scope, 'file> {
    pub(crate) fn claims(
        scope: &'scope Scope<'file>,
        place: &Place,
        block: &'file Block,
        quantifiers: &Quantifiers,
    ) -> Vec<Claim> {
        let mut body = Body {
            scope,
            locals: Locals::default(),
            quantifiers: quantifiers.clone(),
            open: BTreeSet::new(),
            place: place.clone(),
            claims: Vec::new(),
        };
        body.visit_block(block);
        body.claims
    }

    fn met(&mut self, invocation: &Macro) {
        let assertion = Assertion::named(invocation, self.scope.tables());
        let found = assertion.claims(invocation, &Trace::new(self.scope, &self.locals, &self.place, &self.quantifiers));
        self.claims.extend(found);
        assertion.nested(invocation).iter().for_each(|inner| self.met(inner));
    }

    fn called(&mut self, callee: Option<&Path>, arguments: &[&'file Expr]) {
        let scope = self.scope;
        let place = self.place.clone();
        callee
            .map(|path| scope.helpers(path, &place, arguments))
            .unwrap_or_default()
            .iter()
            .for_each(|helper| self.entered(helper, arguments));
    }

    fn entered(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr]) {
        let key = helper.route().clone();
        self.open.insert(key.clone()).then(|| self.expanded(helper, arguments, &key));
    }

    fn expanded(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr], key: &Route) {
        let inner = helper.called(arguments, &self.locals);
        let outer = mem::replace(&mut self.locals, inner);
        let place = mem::replace(&mut self.place, helper.place().clone());
        helper.visited(self);
        self.place = place;
        self.locals = outer;
        self.open.remove(key);
    }
}

impl<'file> Visit<'file> for Body<'_, 'file> {
    fn visit_local(&mut self, local: &'file Local) {
        self.locals.bind(local);
        visit::visit_local(self, local);
    }

    fn visit_block(&mut self, block: &'file Block) {
        block.stmts.iter().find(|statement| {
            self.visit_stmt(statement);
            Reach::after(&Trace::new(self.scope, &self.locals, &self.place, &self.quantifiers), statement) == Flow::Ends
        });
    }

    fn visit_expr_if(&mut self, branch: &'file ExprIf) {
        match Evaluation::condition_of(
            &Trace::new(self.scope, &self.locals, &self.place, &self.quantifiers),
            &branch.cond,
        ) {
            Some(Truth::Holds) => self.visit_block(&branch.then_branch),
            Some(Truth::Fails) => branch.else_branch.iter().for_each(|(_, other)| self.visit_expr(other)),
            None => visit::visit_expr_if(self, branch),
        }
    }

    fn visit_expr_while(&mut self, looped: &'file ExprWhile) {
        match Evaluation::condition_of(
            &Trace::new(self.scope, &self.locals, &self.place, &self.quantifiers),
            &looped.cond,
        ) {
            Some(Truth::Fails) => (),
            Some(Truth::Holds) | None => visit::visit_expr_while(self, looped),
        }
    }

    fn visit_expr_for_loop(&mut self, looped: &'file ExprForLoop) {
        let binder = Iterations::looped(&Trace::new(self.scope, &self.locals, &self.place, &self.quantifiers), looped);
        let inner = self.quantifiers.quantified(binder, &looped.pat);
        let outer = mem::replace(&mut self.quantifiers, inner);
        visit::visit_expr_for_loop(self, looped);
        self.quantifiers = outer;
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }

    fn visit_expr_call(&mut self, call: &'file ExprCall) {
        self.called(Syntax::expr_path(&call.func).map(|path| &path.path), &call.args.iter().collect::<Vec<&Expr>>());
        visit::visit_expr_call(self, call);
    }
}
