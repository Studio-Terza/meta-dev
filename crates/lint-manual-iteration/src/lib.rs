use lint_core::FileLint;
use lint_core::{Headline, Lint};
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
        Report::new(
            Headline::new("iterators are driven by combinators, not by hand"),
            files.iter().filter_map(SourceFile::shipped).flat_map(Self::violations).collect(),
        )
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
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::If(branch), _)] if branch.else_branch.is_none() => Self::accumulator(&branch.then_branch),
            [Stmt::Expr(Expr::MethodCall(call), _)] if matches!(*call.receiver, Expr::Path(_) | Expr::Field(_)) => {
                Name::from(&call.method).matches::<Whole>(&Self::ACCUMULATORS).map(|_| call)
            }
            _ => None,
        }
    }
}

impl FileLint for ManualIteration {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut iterations = Iterations { path: file.path(), violations: Vec::new() };
        iterations.visit_file(file.syntax());
        iterations.violations
    }
}

struct Iterations<'scan> {
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
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_impl_item_fn(self, function));
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let finding = (matches!(&local.pat, Pat::Ident(binding) if binding.mutability.is_some())
            && local.init.as_ref().is_some_and(|init| {
                let mut calls = IteratorCalls::default();
                calls.visit_expr(&init.expr);
                calls.0 > 0
            }))
        .then(|| {
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
