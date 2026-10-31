mod finding;
mod nesting_depth;

use crate::Lint;
use finding::Finding;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Arm, BinOp, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg, ImplItemFn, ItemFn,
    ItemImpl, ItemMod, ItemTrait, Lit, Local, Macro, Pat, Signature, Type, UnOp,
};

use lint_core::LineNumber;
use lint_core::Macros;
use lint_core::Most;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_core::{Name, Prefix, Whole};

use nesting_depth::NestingDepth;

#[derive(Debug)]
pub struct FnShape;

impl Lint for FnShape {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "hairy control flow; booleans and tuples become types, nesting becomes methods",
            files.iter().filter(|file| file.is_shipped()).flat_map(Self::violations).collect(),
        )
    }
}

impl FnShape {
    const BOOLEAN_METHODS: &[&str] = &["contains", "all", "any", "starts_with", "ends_with", "eq", "ne"];
    const BOOLEAN_PREFIXES: [&str; 2] = ["is_", "has_"];
    const NESTING: Most = Most::new(2);
    const PARAMETERS: Most = Most::new(4);

    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut shapes = Shapes { path: file.path(), depth: NestingDepth::default(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
    }

    fn is_boolean(expr: &Expr) -> bool {
        match expr {
            Expr::Binary(binary) => matches!(
                binary.op,
                BinOp::Eq(_)
                    | BinOp::Ne(_)
                    | BinOp::Lt(_)
                    | BinOp::Le(_)
                    | BinOp::Gt(_)
                    | BinOp::Ge(_)
                    | BinOp::And(_)
                    | BinOp::Or(_)
            ),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Not(_)),
            Expr::Lit(literal) => matches!(literal.lit, Lit::Bool(_)),
            Expr::Paren(paren) => Self::is_boolean(&paren.expr),
            Expr::MethodCall(call) => {
                let name = Name::from(&call.method);
                name.matches::<Prefix>(&Self::BOOLEAN_PREFIXES) || name.matches::<Whole>(Self::BOOLEAN_METHODS)
            }
            _ => false,
        }
    }

    fn binds_bool(local: &Local) -> bool {
        matches!(
            &local.pat,
            Pat::Type(typed) if matches!(&*typed.ty, Type::Path(path) if path.path.is_ident("bool"))
        ) || local.init.as_ref().is_some_and(|init| Self::is_boolean(&init.expr))
    }
}

struct Shapes<'scan> {
    path: &'scan Path,
    depth: NestingDepth,
    violations: Vec<Violation>,
}

impl Shapes<'_> {
    fn found(path: &Path, span: Span, finding: Finding) -> Violation {
        Violation::new(Site::new(path, LineNumber::from(span)), finding)
    }

    fn nested(&mut self, span: Span, visit: impl FnOnce(&mut Self)) {
        self.depth += NestingDepth::LEVEL;
        let first_past = NestingDepth::beyond(FnShape::NESTING);
        let deepest = (self.depth == first_past).then_some(span);
        self.violations
            .extend(deepest.map(|span| Self::found(self.path, span, Finding::DeepNesting { most: FnShape::NESTING })));
        visit(self);
        self.depth -= NestingDepth::LEVEL;
    }
}

impl<'ast> Visit<'ast> for Shapes<'_> {
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

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let wide = (signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count()
            > FnShape::PARAMETERS.count())
        .then_some(signature.ident.span());
        self.violations.extend(
            wide.map(|span| Self::found(self.path, span, Finding::WideSignature { most: FnShape::PARAMETERS })),
        );
        syn::visit::visit_signature(self, signature);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        self.violations.extend(
            FnShape::binds_bool(local)
                .then_some(local.span())
                .map(|span| Self::found(self.path, span, Finding::BooleanLocal)),
        );
        self.violations.extend(
            local
                .init
                .as_ref()
                .filter(|init| matches!(*init.expr, Expr::Tuple(_)))
                .map(|_| Self::found(self.path, local.span(), Finding::TupleLocal)),
        );
        syn::visit::visit_local(self, local);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.violations.extend(
            call.args
                .first()
                .filter(|seed| matches!(seed, Expr::Tuple(_)))
                .filter(|_| call.method == "fold" || call.method == "try_fold")
                .map(|_| Self::found(self.path, call.method.span(), Finding::TupleAccumulator)),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.visit_expr(&expr.cond);
        self.nested(expr.if_token.span, |this| {
            this.visit_block(&expr.then_branch);
        });
        match expr.else_branch.as_ref().map(|(_, branch)| &**branch) {
            None => {}
            Some(Expr::If(chained)) => self.visit_expr_if(chained),
            Some(other) => self.nested(other.span(), |this| this.visit_expr(other)),
        }
    }

    fn visit_arm(&mut self, arm: &'ast Arm) {
        self.visit_pat(&arm.pat);
        arm.guard.iter().map(|(_, guard)| guard).for_each(|guard| self.visit_expr(guard));
        self.nested(arm.body.span(), |this| this.visit_expr(&arm.body));
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.visit_expr(&expr.expr);
        self.nested(expr.for_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.visit_expr(&expr.cond);
        self.nested(expr.while_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.nested(expr.loop_token.span, |this| this.visit_block(&expr.body));
    }

    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        self.nested(closure.or1_token.span, |this| {
            this.visit_expr(&closure.body);
        });
    }
}
