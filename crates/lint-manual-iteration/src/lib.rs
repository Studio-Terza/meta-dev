use lint_core::Lint;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole};

#[derive(Debug)]
pub struct ManualIteration;

impl Lint for ManualIteration {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "iterators are driven by combinators, not by hand",
            files.iter().filter(|file| file.is_shipped()).flat_map(Self::violations).collect(),
        )
    }
}

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [&str; 16] = [
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
    const ACCUMULATORS: [&str; 4] = ["push", "extend", "insert", "push_str"];

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut iterations = Iterations { path: file.path(), violations: Vec::new() };
        iterations.visit_file(file.syntax());
        iterations.violations
    }

    fn accumulates(block: &Block) -> bool {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::If(branch), _)] if branch.else_branch.is_none() => Self::accumulates(&branch.then_branch),
            [Stmt::Expr(Expr::MethodCall(call), _)] => {
                Name::from(&call.method).matches::<Whole>(&Self::ACCUMULATORS)
                    && matches!(*call.receiver, Expr::Path(_) | Expr::Field(_))
            }
            _ => false,
        }
    }
}

struct Iterations<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl Iterations<'_> {
    fn violation(&self, span: Span, message: &str) -> Violation {
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
                "mutable iterator local; drive it with combinators such as format, collect, or fold",
            )
        });
        self.violations.extend(finding);
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        let finding = ManualIteration::accumulates(&expr.body).then(|| {
            self.violation(
                expr.for_token.span,
                "loop that only accumulates; extend or collect from an iterator instead",
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
        self.0 += usize::from(Name::from(&call.method).matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS));
        syn::visit::visit_expr_method_call(self, call);
    }
}
