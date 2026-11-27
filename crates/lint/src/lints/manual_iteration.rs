use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use crate::expansion::Expansion;
use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug, Default)]
pub(crate) struct ManualIteration {
    found: Vec<Found>,
}

#[derive(Debug, Default)]
struct Constructed {
    calls: usize,
}

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [&'static str; 16] = [
        "iter",
        "iter_mut",
        "into_iter",
        "chars",
        "bytes",
        "lines",
        "split",
        "split_whitespace",
        "rev",
        "skip",
        "take",
        "zip",
        "enumerate",
        "map",
        "filter",
        "peekable",
    ];
    const ACCUMULATORS: [&'static str; 4] = ["push", "extend", "insert", "push_str"];

    fn accumulator(block: &Block) -> Option<&ExprMethodCall> {
        match block.stmts.as_slice() {
            [only] => ManualIteration::accumulating(only),
            _ => None,
        }
    }

    fn accumulating(statement: &Stmt) -> Option<&ExprMethodCall> {
        match statement {
            Stmt::Expr(Expr::If(branch), _) => match branch.else_branch {
                None => ManualIteration::accumulator(&branch.then_branch),
                Some(_) => None,
            },
            Stmt::Expr(Expr::MethodCall(call), _) => Some(call)
                .filter(|candidate| matches!(*candidate.receiver, Expr::Path(_) | Expr::Field(_)))
                .filter(|on_place| ManualIteration::ACCUMULATORS.iter().any(|name| on_place.method == name)),
            _ => None,
        }
    }

    fn mutable_init(local: &Local) -> Option<&Expr> {
        let mutable = match &local.pat {
            Pat::Ident(binding) => binding.mutability.map(|_| local),
            _ => None,
        };
        mutable.and_then(|binding| binding.init.as_ref()).map(|init| &*init.expr)
    }
}

impl<'ast> Visit<'ast> for ManualIteration {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Gate::of(&item.attrs).entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        Gate::of(&item.attrs).entered().map_or_default(|()| visit::visit_item_trait(self, item));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Gate::of(&function.attrs).entered().map_or_default(|()| visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Gate::of(&function.attrs).entered().map_or_default(|()| visit::visit_impl_item_fn(self, function));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.found.extend(ManualIteration::accumulator(&expr.body).map(|_| {
            Found::spanned(
                expr.for_token.span,
                "loop that only accumulates",
                "extend or collect from an iterator instead",
            )
        }));
        visit::visit_expr_for_loop(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let constructed = ManualIteration::mutable_init(local).filter(|init| {
            let mut calls = Constructed::default();
            calls.visit_expr(init);
            calls.calls > 0
        });
        self.found.extend(constructed.map(|_| {
            Found::spanned(
                local.span(),
                "mutable iterator local",
                "drive it with combinators such as format, collect, or fold",
            )
        }));
        visit::visit_local(self, local);
    }
}

impl<'ast> Visit<'ast> for Constructed {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let constructs = ManualIteration::ITERATOR_CONSTRUCTORS.iter().any(|name| call.method == name);
        self.calls += usize::from(constructs);
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Expansion::of(invocation).walk(self);
    }
}

impl<'file> Style<'file> for ManualIteration {
    const RULE: Rule = Rule::ManualIteration;
    const READS: Reads = Reads::Shipped;

    fn new(_: &'file Source) -> ManualIteration {
        ManualIteration::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
