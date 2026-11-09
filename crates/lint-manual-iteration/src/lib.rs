use lint_core::{Headline, Lint};
use lint_file::FileLint;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use lint_core::Fix;
use lint_core::Macros;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct ManualIteration;

impl Lint for ManualIteration {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(Headline::new("iterators are driven by combinators, not by hand"), Self::in_shipped(files))
    }
}

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

impl FileLint for ManualIteration {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut iterations = Iterations { file, path: file.path(), violations: Vec::new() };
        iterations.visit_file(file.syntax());
        iterations.violations
    }
}

struct Iterations<'scan> {
    file: &'scan SourceFile,
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> Iterations<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }
}

impl<'ast> Visit<'ast> for Iterations<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Walk::enter(&module.attrs, || syn::visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Walk::enter(&item.attrs, || syn::visit::visit_item_impl(self, item));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        Walk::enter(&item.attrs, || syn::visit::visit_item_trait(self, item));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_impl_item_fn(self, function));
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let finding = Some(local)
            .filter(|candidate| matches!(&candidate.pat, Pat::Ident(binding) if binding.mutability.is_some()))
            .and_then(|mutable| mutable.init.as_ref())
            .filter(|init| {
                let mut calls = IteratorCalls::default();
                calls.visit_expr(&init.expr);
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
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
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
        syn::visit::visit_expr_for_loop(self, expr);
    }
}

#[derive(Default)]
struct IteratorCalls(usize);

impl<'ast> Visit<'ast> for IteratorCalls {
    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.0 +=
            Name::from(&call.method).matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS).into_iter().count();
        syn::visit::visit_expr_method_call(self, call);
    }
}
