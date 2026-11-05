use syn::visit::Visit;
use syn::{BinOp, Expr, ExprBinary, ExprCall, ExprMethodCall, Stmt};

use lint_layout::Name;

use super::super::trace::{Locals, Lowering, Trace};
use super::Run;
use crate::boundary::Truth;
use crate::claim::{Callee, Premise, Term};

pub(crate) struct Mutations;

impl Mutations {
    pub(crate) fn compound(operator: &BinOp) -> Option<()> {
        match operator {
            BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_) => Some(()),
            _ => None,
        }
    }

    pub(crate) fn lending(argument: &Expr) -> Option<&Expr> {
        match argument {
            Expr::Reference(reference) => reference.mutability.map(|_| &*reference.expr),
            _ => None,
        }
    }

    pub(crate) fn discarding(statement: &Stmt) -> Option<&ExprMethodCall> {
        match statement {
            Stmt::Expr(Expr::MethodCall(call), Some(_)) => Some(call),
            _ => None,
        }
    }

    pub(crate) fn short(operator: &BinOp) -> Option<Truth> {
        match operator {
            BinOp::And(_) => Some(Truth::Holds),
            BinOp::Or(_) => Some(Truth::Fails),
            _ => None,
        }
    }
}

impl<'file> Run<'_, 'file> {
    pub(crate) fn mutated(&mut self, target: &Expr, term: &Term) {
        let root = Locals::rooted(target).and_then(|root| Locals::named(root).map(|name| (root, name)));
        root.filter(|(_, name)| self.frame.locals().mutable(name).is_some())
            .into_iter()
            .for_each(|(root, name)| self.restated(target, root, name, term));
    }

    pub(crate) fn restated(&mut self, target: &Expr, root: &Expr, name: Name, term: &Term) {
        let previous = Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), root);
        let updated = Callee::Function(Name::from("updated")).folded(vec![previous, term.clone()]);
        let state = Locals::named(target).map_or(updated, |_| term.clone());
        self.frame.restated(name, state);
    }

    pub(crate) fn mutated_by(&mut self, statement: &Stmt) {
        let call = Mutations::discarding(statement);
        let term = call.map(|call| {
            Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &Expr::MethodCall(call.clone()))
        });
        call.zip(term).map(|(call, term)| self.mutated(&call.receiver, &term)).unwrap_or_default();
    }

    pub(crate) fn lent(&mut self, call: &ExprCall) {
        let targets: Vec<&Expr> = call.args.iter().filter_map(Mutations::lending).collect();
        let term = targets
            .first()
            .map(|_| Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &Expr::Call(call.clone())));
        targets.iter().zip(term.iter().cycle()).for_each(|(target, term)| self.mutated(target, term));
    }

    pub(crate) fn short_circuited(&mut self, binary: &'file ExprBinary) {
        let held = Mutations::short(&binary.op).map(|holds| {
            Premise::new(Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), &binary.left), holds)
        });
        self.visited_under(held.as_ref(), &binary.right);
    }

    pub(crate) fn visited_under(&mut self, held: Option<&Premise>, expression: &'file Expr) {
        match held {
            Some(premise) => self.conditioned(premise, |run| run.visit_expr(expression)),
            None => self.visit_expr(expression),
        }
    }
}
