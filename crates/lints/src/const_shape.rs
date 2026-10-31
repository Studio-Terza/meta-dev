use proc_macro2::Span;
use std::path::Path;
use syn::visit::Visit;

use syn::{
    Expr, ExprCast, ExprConst, ExprForLoop, ExprLoop, ExprRepeat, ExprWhile, ImplItemConst, ImplItemFn, ItemConst,
    ItemFn, ItemMod, ItemStatic, Macro, TraitItemConst, TraitItemFn, TypeArray, Variant,
};

use lint_core::LineNumber;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

#[derive(Debug)]
pub struct ConstShape;

impl ConstShape {
    #[must_use]
    pub fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "casts live in const fns, and const loops walk slices",
            files.iter().flat_map(Self::violations).collect(),
        )
    }

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut contexts = ConstContexts { path: file.path(), context: Context::Surface, violations: Vec::new() };
        contexts.visit_file(file.syntax());
        contexts.violations
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Surface,
    Const,
}

impl Context {
    fn of_fn(constness: Option<&syn::token::Const>) -> Context {
        constness.map_or(Context::Surface, |_| Context::Const)
    }
}

struct ConstContexts<'scan> {
    path: &'scan Path,
    context: Context,
    violations: Vec<Violation>,
}

impl ConstContexts<'_> {
    fn violation(&self, span: Span, message: &str) -> Violation {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn within(&mut self, context: Context, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.context, context);
        visit(self);
        self.context = outer;
    }

    fn cast_finding(&self, cast: &ExprCast) -> Option<Violation> {
        (self.context == Context::Surface)
            .then(|| self.violation(cast.as_token.span, "as cast outside a const fn; use From, TryFrom, or an EnumMap"))
    }

    fn loop_finding(&self, span: Span, message: &str) -> Option<Violation> {
        (self.context == Context::Const).then(|| self.violation(span, message))
    }
}

impl<'ast> Visit<'ast> for ConstContexts<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        let context = Context::of_fn(function.sig.constness.as_ref());
        self.within(context, |this| syn::visit::visit_item_fn(this, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        let context = Context::of_fn(function.sig.constness.as_ref());
        self.within(context, |this| syn::visit::visit_impl_item_fn(this, function));
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.within(Context::Surface, |this| syn::visit::visit_trait_item_fn(this, function));
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.within(Context::Const, |this| syn::visit::visit_item_const(this, item));
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.within(Context::Const, |this| syn::visit::visit_item_static(this, item));
    }

    fn visit_impl_item_const(&mut self, item: &'ast ImplItemConst) {
        self.within(Context::Const, |this| syn::visit::visit_impl_item_const(this, item));
    }

    fn visit_trait_item_const(&mut self, item: &'ast TraitItemConst) {
        self.within(Context::Const, |this| syn::visit::visit_trait_item_const(this, item));
    }

    fn visit_expr_const(&mut self, block: &'ast ExprConst) {
        self.within(Context::Const, |this| syn::visit::visit_expr_const(this, block));
    }

    fn visit_type_array(&mut self, array: &'ast TypeArray) {
        self.visit_type(&array.elem);
        self.within(Context::Const, |this| this.visit_expr(&array.len));
    }

    fn visit_expr_repeat(&mut self, repeat: &'ast ExprRepeat) {
        self.visit_expr(&repeat.expr);
        self.within(Context::Const, |this| this.visit_expr(&repeat.len));
    }

    fn visit_variant(&mut self, variant: &'ast Variant) {
        self.visit_fields(&variant.fields);
        if let Some((_, discriminant)) = &variant.discriminant {
            self.within(Context::Const, |this| this.visit_expr(discriminant));
        }
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        for statement in Macros::statements(invocation) {
            self.visit_stmt(&statement);
        }
    }

    fn visit_expr_cast(&mut self, cast: &'ast ExprCast) {
        let finding = self.cast_finding(cast);
        self.violations.extend(finding);
        syn::visit::visit_expr_cast(self, cast);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        let finding = (!matches!(*expr.cond, Expr::Let(_))).then(|| {
            self.loop_finding(
                expr.while_token.span,
                "counter loop in const context; walk a slice with while let [head, rest @ ..]",
            )
        });
        self.violations.extend(finding.flatten());
        syn::visit::visit_expr_while(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        let finding = self
            .loop_finding(expr.loop_token.span, "open loop in const context; walk a slice with while let or recurse");
        self.violations.extend(finding);
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        let finding = self.loop_finding(expr.for_token.span, "for loop in const context");
        self.violations.extend(finding);
        syn::visit::visit_expr_for_loop(self, expr);
    }
}
