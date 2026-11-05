mod attributes;
mod closures;
mod flow;
mod mutations;

use std::collections::BTreeSet;
use std::mem;

use syn::visit::{self, Visit};
use syn::{
    Arm, Block, Expr, ExprAssign, ExprAsync, ExprBinary, ExprCall, ExprClosure, ExprForLoop, ExprIf, ExprMatch,
    ExprMethodCall, ExprTry, ExprWhile, Item, Local, Macro, Path, Stmt,
};

use lint_layout::Name;

use super::assertion::Assertion;
use super::scope::{Helper, Place, Route, Scope};
use super::trace::{Evaluation, Frame, Lowering, Quantifiers, Trace};
use crate::boundary::{Spelling, Truth};
use crate::claim::{Accepted, Callee, Claim, Guard, Premise, Term};
pub(crate) use attributes::Attributes;
use flow::{Flow, Reach};
use mutations::Mutations;

pub(crate) struct Execution<'run, 'file> {
    block: &'file Block,
    quantifiers: &'run Quantifiers,
    attributes: Attributes<'run>,
}

impl<'run, 'file> Execution<'run, 'file> {
    pub(crate) fn new(
        block: &'file Block,
        quantifiers: &'run Quantifiers,
        attributes: Attributes<'run>,
    ) -> Execution<'run, 'file> {
        Execution { block, quantifiers, attributes }
    }
}

pub(crate) struct Run<'scope, 'file> {
    scope: &'scope Scope<'file>,
    frame: Frame<'file>,
    open: BTreeSet<Route>,
    place: Place,
    claims: Vec<Claim>,
}

impl<'scope, 'file> Run<'scope, 'file> {
    pub(crate) fn claims(scope: &'scope Scope<'file>, place: &Place, execution: &Execution<'_, 'file>) -> Vec<Claim> {
        let attributes = &execution.attributes;
        let executing = attributes.panicking().map_or_else(|| attributes.guard(), |_| Guard::falsified());
        let mut run = Run {
            scope,
            frame: Frame::seeded(execution.quantifiers, executing),
            open: BTreeSet::new(),
            place: place.clone(),
            claims: Vec::new(),
        };
        run.visit_block(execution.block);
        run.claims.into_iter().chain(attributes.expecting(execution.block, execution.quantifiers.binders())).collect()
    }

    pub(crate) fn nested(&mut self, change: impl FnOnce(&mut Frame<'file>), body: impl FnOnce(&mut Self)) {
        let outer = self.frame.clone();
        change(&mut self.frame);
        body(self);
        let inner = mem::replace(&mut self.frame, outer);
        self.frame.settled(&inner);
    }

    pub(crate) fn conditioned(&mut self, premise: &Premise, body: impl FnOnce(&mut Self)) {
        self.nested(|frame| premise.clone().conjuncts().iter().for_each(|part| frame.assumed(part)), body);
    }

    pub(crate) fn stated(&mut self, subject: &Expr, accepted: Accepted) {
        let found = Trace::new(self.scope, &self.place, &self.frame).stated(subject, accepted);
        self.claims.extend(found);
    }

    pub(crate) fn met(&mut self, invocation: &Macro) {
        let assertion = Assertion::named(invocation, self.scope.tables());
        let found = assertion.claims(invocation, &Trace::new(self.scope, &self.place, &self.frame));
        self.claims.extend(found);
        assertion.nested(invocation).iter().for_each(|inner| self.met(inner));
    }

    pub(crate) fn called(&mut self, callee: Option<&Path>, arguments: &[&'file Expr]) {
        let scope = self.scope;
        let place = self.place.clone();
        callee
            .map(|path| scope.helpers(path, &place, arguments))
            .unwrap_or_default()
            .iter()
            .for_each(|helper| self.entered(helper, arguments));
    }

    pub(crate) fn entered(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr]) {
        let key = helper.route().clone();
        self.open.insert(key.clone()).then(|| self.expanded(helper, arguments, &key));
    }

    pub(crate) fn expanded(&mut self, helper: &Helper<'file>, arguments: &[&'file Expr], key: &Route) {
        let inner = helper.called(arguments, self.frame.locals());
        let outer = mem::replace(self.frame.locals_mut(), inner);
        let place = mem::replace(&mut self.place, helper.place().clone());
        helper.visited(self);
        self.place = place;
        self.frame.localized(outer);
        self.open.remove(key);
    }

    pub(crate) fn step(&mut self, statement: &'file Stmt) -> Flow {
        self.visit_stmt(statement);
        let trace = Trace::new(self.scope, &self.place, &self.frame);
        let flow = Reach::after(&trace, statement);
        let remaining = Reach::remaining(&trace, statement);
        let guard = remaining.into_iter().fold(self.frame.guard().clone(), |guard, premise| guard.conjoined(premise));
        self.frame.guarded(guard);
        flow
    }

    pub(crate) fn undecided(&mut self, branch: &'file ExprIf) {
        self.visit_expr(&branch.cond);
        let term = Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &branch.cond);
        self.conditioned(&Premise::new(term.clone(), Truth::Holds), |run| run.visit_block(&branch.then_branch));
        branch.else_branch.iter().for_each(|(_, other)| {
            self.conditioned(&Premise::new(term.clone(), Truth::Fails), |run| run.visit_expr(other));
        });
    }

    pub(crate) fn waiting(&mut self, looped: &'file ExprWhile) {
        let term = Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &looped.cond);
        self.visit_expr(&looped.cond);
        self.conditioned(&Premise::new(term, Truth::Holds), |run| run.visit_block(&looped.body));
    }

    pub(crate) fn arm(&mut self, scrutinee: &Term, arm: &'file Arm) {
        let spelled = Term::Opaque(Spelling::written(&arm.pat));
        let guarded = Term::Opaque(Spelling::written(&arm.guard.as_ref().map(|(_, guard)| guard)));
        let premise = Premise::new(
            Callee::Function(Name::from("matches")).folded(vec![scrutinee.clone(), spelled, guarded]),
            Truth::Holds,
        );
        self.conditioned(&premise, |run| run.visit_expr(&arm.body));
    }
}

impl<'file> Visit<'file> for Run<'_, 'file> {
    fn visit_item(&mut self, _item: &'file Item) {}

    fn visit_stmt(&mut self, statement: &'file Stmt) {
        let outer = self.frame.guard().clone();
        self.frame.guarded(outer.clone().joined(Attributes::of_statement(statement).guard()));
        visit::visit_stmt(self, statement);
        self.mutated_by(statement);
        self.frame.guarded(outer);
    }

    fn visit_local(&mut self, local: &'file Local) {
        self.frame.locals_mut().bind(local);
        visit::visit_local(self, local);
    }

    fn visit_block(&mut self, block: &'file Block) {
        let outer = self.frame.guard().clone();
        block.stmts.iter().find(|statement| self.step(statement) != Flow::Falls);
        self.frame.guarded(outer);
    }

    fn visit_expr_if(&mut self, branch: &'file ExprIf) {
        match Evaluation::condition_of(&Trace::new(self.scope, &self.place, &self.frame), &branch.cond) {
            Some(Truth::Holds) => self.visit_block(&branch.then_branch),
            Some(Truth::Fails) => branch.else_branch.iter().for_each(|(_, other)| self.visit_expr(other)),
            None => self.undecided(branch),
        }
    }

    fn visit_expr_match(&mut self, matched: &'file ExprMatch) {
        self.visit_expr(&matched.expr);
        let scrutinee = Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &matched.expr);
        matched.arms.iter().for_each(|arm| self.arm(&scrutinee, arm));
    }

    fn visit_expr_while(&mut self, looped: &'file ExprWhile) {
        match Evaluation::condition_of(&Trace::new(self.scope, &self.place, &self.frame), &looped.cond) {
            Some(Truth::Fails) => (),
            Some(Truth::Holds) => self.nested(|_| (), |run| run.visit_block(&looped.body)),
            None => self.waiting(looped),
        }
    }

    fn visit_expr_for_loop(&mut self, looped: &'file ExprForLoop) {
        self.visit_expr(&looped.expr);
        let leaving = Reach::within(&Trace::new(self.scope, &self.place, &self.frame), &looped.body).only(Flow::Leaves);
        self.iterating(&looped.expr, &looped.pat, leaving, |run| run.visit_block(&looped.body));
    }

    fn visit_expr_closure(&mut self, _closure: &'file ExprClosure) {}

    fn visit_expr_async(&mut self, block: &'file ExprAsync) {
        let premise = Premise::new(Term::Opaque(Spelling::written(block)), Truth::Holds);
        self.conditioned(&premise, |run| visit::visit_expr_async(run, block));
    }

    fn visit_expr_binary(&mut self, binary: &'file ExprBinary) {
        self.visit_expr(&binary.left);
        self.short_circuited(binary);
        Mutations::compound(&binary.op)
            .map(|()| {
                Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &Expr::Binary(binary.clone()))
            })
            .map(|term| self.mutated(&binary.left, &term))
            .unwrap_or_default();
    }

    fn visit_expr_assign(&mut self, assignment: &'file ExprAssign) {
        visit::visit_expr_assign(self, assignment);
        let term = Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &assignment.right);
        self.mutated(&assignment.left, &term);
    }

    fn visit_expr_try(&mut self, attempt: &'file ExprTry) {
        visit::visit_expr_try(self, attempt);
        self.stated(&attempt.expr, Accepted::unwrapped(Accepted::Any));
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }

    fn visit_expr_call(&mut self, call: &'file ExprCall) {
        self.invoked(call, &call.args.iter().collect::<Vec<&'file Expr>>());
    }

    fn visit_expr_method_call(&mut self, call: &'file ExprMethodCall) {
        self.applied(call);
    }
}
