use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprClosure, ExprMethodCall, Pat};

use lint_core::Syntax;
use lint_layout::{Name, Whole, Word, words};

use super::super::trace::{Frame, Iterations, Locals, Lowering, Trace};
use super::Run;
use super::flow::Flow;
use crate::boundary::Truth;
use crate::claim::{Accepted, Binder, Guard, Premise};

pub(crate) struct Closures;

impl Closures {
    const SWALLOWERS: [Word<'static>; 1] = words!["catch_unwind"];
    const UNWRAPPERS: [Word<'static>; 2] = words!["unwrap", "expect"];
    const LOOPERS: [Word<'static>; 2] = words!["for_each", "try_for_each"];

    pub(crate) fn within(expression: &Expr) -> Option<&ExprClosure> {
        match expression {
            Expr::Closure(closure) => Some(closure),
            Expr::Paren(inner) => Closures::within(&inner.expr),
            _ => None,
        }
    }

    pub(crate) fn calling(expression: &Expr) -> Option<&ExprCall> {
        match expression {
            Expr::Call(call) => Some(call),
            _ => None,
        }
    }

    pub(crate) fn swallowing(call: &ExprCall) -> Option<&ExprClosure> {
        Syntax::expr_path(&call.func)
            .and_then(|path| path.path.segments.last())
            .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::SWALLOWERS))
            .and_then(|_| call.args.first())
            .and_then(Closures::within)
    }

    pub(crate) fn unwrapping(call: &ExprMethodCall) -> Option<Name> {
        Some(Name::from(&call.method)).filter(|name| name.matches::<Whole>(&Self::UNWRAPPERS).is_some())
    }

    pub(crate) fn looping(call: &ExprMethodCall) -> Option<&ExprClosure> {
        Some(Name::from(&call.method))
            .filter(|name| name.matches::<Whole>(&Self::LOOPERS).is_some())
            .and_then(|_| call.args.first())
            .and_then(Closures::within)
    }

    pub(crate) fn bound<'file>(locals: Locals<'file>, input: &Pat, value: &'file Expr) -> Locals<'file> {
        match Closures::named(input) {
            Some(name) => locals.binding(name, value),
            None => locals,
        }
    }

    pub(crate) fn named(pattern: &Pat) -> Option<Name> {
        match pattern {
            Pat::Ident(named) => Some(Name::from(&named.ident)),
            Pat::Type(typed) => Closures::named(&typed.pat),
            Pat::Reference(reference) => Closures::named(&reference.pat),
            _ => None,
        }
    }
}

impl<'file> Run<'_, 'file> {
    pub(crate) fn iterating(
        &mut self,
        iterable: &Expr,
        pattern: &Pat,
        leaving: Option<Flow>,
        body: impl FnOnce(&mut Self),
    ) {
        let every = Iterations::over(&Trace::new(self.scope, &self.place, &self.frame), iterable);
        let binder: Binder = leaving.map_or_else(|| every.clone(), |_| every.first());
        self.nested(|frame| frame.quantified(binder, pattern), body);
    }

    pub(crate) fn inlined(&mut self, closure: &'file ExprClosure, arguments: &[&'file Expr]) {
        let caller = self.frame.locals().clone();
        let inner = closure.inputs.iter().zip(arguments).fold(caller.clone(), |locals, (input, argument)| {
            Closures::bound(locals, input, caller.resolved(argument))
        });
        self.nested(|frame: &mut Frame<'file>| frame.localized(inner), |run| run.visit_expr(&closure.body));
    }

    pub(crate) fn executed(
        &mut self,
        local: Option<&'file ExprClosure>,
        call: &'file ExprCall,
        arguments: &[&'file Expr],
    ) {
        match local {
            Some(closure) => self.inlined(closure, arguments),
            None => self.called(Syntax::expr_path(&call.func).map(|path| &path.path), arguments),
        }
    }

    pub(crate) fn passed(&mut self, callee: &Expr, closure: &'file ExprClosure, swallowed: Option<&ExprClosure>) {
        let premise =
            Premise::new(Lowering::term_of(&Trace::new(self.scope, &self.place, &self.frame), callee), Truth::Holds);
        self.sealed(swallowed, &premise, closure);
    }

    pub(crate) fn sealed(&mut self, swallowed: Option<&ExprClosure>, premise: &Premise, closure: &'file ExprClosure) {
        match swallowed {
            Some(_) => self.nested(|frame| frame.guarded(Guard::falsified()), |run| run.visit_expr(&closure.body)),
            None => self.conditioned(premise, |run| run.visit_expr(&closure.body)),
        }
    }

    pub(crate) fn invoked(&mut self, call: &'file ExprCall, arguments: &[&'file Expr]) {
        let local = Closures::within(self.frame.locals().resolved(&call.func));
        self.executed(local, call, arguments);
        self.visit_expr(&call.func);
        let swallowing = Closures::swallowing(call);
        let callee = Expr::Call(call.clone());
        arguments.iter().copied().for_each(|argument| self.passing(&callee, swallowing, argument));
        self.lent(call);
    }

    pub(crate) fn passing(&mut self, callee: &Expr, swallowing: Option<&ExprClosure>, argument: &'file Expr) {
        match Closures::within(argument) {
            Some(closure) => self.passed(callee, closure, swallowing),
            None => self.visit_expr(argument),
        }
    }

    pub(crate) fn receiving(&mut self, propagated: Option<&'file ExprClosure>, receiver: &'file Expr) {
        match propagated {
            Some(closure) => self.inlined(closure, &[]),
            None => self.visit_expr(receiver),
        }
    }

    pub(crate) fn applied(&mut self, call: &'file ExprMethodCall) {
        let unwrapping = Closures::unwrapping(call);
        let propagated =
            unwrapping.as_ref().and_then(|_| Closures::calling(&call.receiver)).and_then(Closures::swallowing);
        self.receiving(propagated, &call.receiver);
        let callee = Expr::MethodCall(call.clone());
        let looping = Closures::looping(call);
        call.args.iter().for_each(|argument| self.arguing(&callee, call, looping, argument));
        unwrapping
            .map(|_| &call.receiver)
            .into_iter()
            .for_each(|receiver| self.stated(receiver, Accepted::unwrapped(Accepted::Any)));
    }

    pub(crate) fn arguing(
        &mut self,
        callee: &Expr,
        call: &'file ExprMethodCall,
        looping: Option<&ExprClosure>,
        argument: &'file Expr,
    ) {
        match Closures::within(argument) {
            Some(closure) => self.argued(callee, call, closure, looping),
            None => self.visit_expr(argument),
        }
    }

    pub(crate) fn argued(
        &mut self,
        callee: &Expr,
        call: &'file ExprMethodCall,
        closure: &'file ExprClosure,
        looping: Option<&ExprClosure>,
    ) {
        match closure.inputs.first().zip(looping) {
            Some((input, _)) => self.iterating(&call.receiver, input, None, |run| run.visit_expr(&closure.body)),
            None => self.passed(callee, closure, None),
        }
    }
}
