use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Sweep, Watch};
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;

use syn::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct ManualIteration;

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [Word<'static>; 16] = words![
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
    const ACCUMULATORS: [Word<'static>; 4] = words!["push", "extend", "insert", "push_str"];

    fn accumulator(block: &Block) -> Option<&ExprMethodCall> {
        Some(block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Self::accumulating)
    }

    fn accumulating(statement: &Stmt) -> Option<&ExprMethodCall> {
        match statement {
            Stmt::Expr(Expr::If(branch), _) => Some(branch)
                .filter(|candidate| candidate.else_branch.is_none())
                .and_then(|bare_if| Self::accumulator(&bare_if.then_branch)),
            Stmt::Expr(Expr::MethodCall(call), _) => Self::accumulated(call),
            _ => None,
        }
    }

    fn accumulated(call: &ExprMethodCall) -> Option<&ExprMethodCall> {
        Some(call)
            .filter(|candidate| matches!(*candidate.receiver, Expr::Path(_) | Expr::Field(_)))
            .and_then(|on_place| Name::from(&on_place.method).matches::<Whole>(&Self::ACCUMULATORS).map(|_| on_place))
    }
}

impl SweptLint for ManualIteration {
    const HEADLINE: Headline = Headline::new("iterators are driven by combinators, not by hand");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Iterations { path: file.head().path(), violations: Vec::new() })
    }
}

#[derive(Debug)]
struct Iterations<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> Iterations<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }
}

impl<'ast> Watch<'ast> for Iterations<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::Local,
            Kind::ExprForLoop,
        ]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_item_trait(&mut self, item: &'ast ItemTrait) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        Descend::from(Walk::outside_tests(&function.attrs))
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        let finding = Some(local)
            .filter(|candidate| matches!(&candidate.pat, Pat::Ident(binding) if binding.mutability.is_some()))
            .and_then(|mutable| mutable.init.as_ref())
            .filter(|init| {
                let mut calls = IteratorCalls::default();
                Sweep::through_expr(&init.expr, &mut calls);
                calls.0 > 0
            })
            .map(|_| {
                self.violation(
                    local.span(),
                    Message::new(
                        Problem::new("mutable iterator local"),
                        Fix::new("drive it with combinators such as format, collect, or fold"),
                    ),
                )
            });
        self.violations.extend(finding);
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        let finding = ManualIteration::accumulator(&expr.body).map(|_| {
            self.violation(
                expr.for_token.span,
                Message::new(
                    Problem::new("loop that only accumulates"),
                    Fix::new("extend or collect from an iterator instead"),
                ),
            )
        });
        self.violations.extend(finding);
        Descend::Into
    }
}

impl<'file> Finds<'file> for Iterations<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}

#[derive(Default)]
struct IteratorCalls(usize);

impl<'ast> Watch<'ast> for IteratorCalls {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ExprMethodCall]
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        self.0 +=
            Name::from(&call.method).matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS).into_iter().count();
        Descend::Into
    }
}
