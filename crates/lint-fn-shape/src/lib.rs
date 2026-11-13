mod finding;
mod nesting_depth;

use finding::Finding;
use lint_core::{Headline, Lint};
use lint_file::{FileLint, Reads};
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Arm, BinOp, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMethodCall, ExprWhile, FnArg, ImplItemFn, ItemFn,
    ItemImpl, ItemMod, ItemTrait, Lit, Local, Macro, Pat, Signature, UnOp,
};

use lint_core::Bound;
use lint_core::Macros;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::{Length, Name, Prefix, Whole, Word, words};

use nesting_depth::NestingDepth;

#[derive(Debug)]
pub struct FnShape;

impl Lint for FnShape {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FnShape {
    const BOOLEAN_METHODS: &[Word<'static>] = &words!["contains", "all", "any", "starts_with", "ends_with", "eq", "ne"];
    const BOOLEAN_PREFIXES: [Word<'static>; 2] = words!["is_", "has_"];
    const NESTING: Bound = Bound::Most(2);
    const PARAMETERS: Bound = Bound::Most(4);
    const FOLDS: [&'static str; 2] = ["fold", "try_fold"];

    fn boolean(expr: &Expr) -> Option<&Expr> {
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
            )
            .then_some(expr),
            Expr::Unary(unary) => matches!(unary.op, UnOp::Not(_)).then_some(expr),
            Expr::Lit(literal) => matches!(literal.lit, Lit::Bool(_)).then_some(expr),
            Expr::Paren(paren) => Self::boolean(&paren.expr).map(|_| expr),
            Expr::MethodCall(call) => Self::boolean_method(&Name::from(&call.method)).map(|_| expr),
            _ => None,
        }
    }

    fn boolean_method(name: &Name) -> Option<Word<'static>> {
        name.matches::<Prefix>(&Self::BOOLEAN_PREFIXES).or_else(|| name.matches::<Whole>(Self::BOOLEAN_METHODS))
    }

    fn binds_bool(local: &Local) -> Option<&Local> {
        Self::typed_bool(&local.pat)
            .map(|_| local)
            .or_else(|| local.init.as_ref().and_then(|init| Self::boolean(&init.expr)).map(|_| local))
    }

    fn typed_bool(pattern: &Pat) -> Option<&Pat> {
        match pattern {
            Pat::Type(typed) => Syntax::type_path(&typed.ty).filter(|path| path.path.is_ident("bool")).map(|_| pattern),
            _ => None,
        }
    }
}

impl FileLint for FnShape {
    const HEADLINE: Headline =
        Headline::new("hairy control flow; booleans and tuples become types, nesting becomes methods");
    const READS: Reads = Reads::Shipped;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut shapes =
            Shapes { file, path: file.head().path(), depth: NestingDepth::default(), violations: Vec::new() };
        shapes.visit_file(file.syntax());
        shapes.violations
    }
}

struct Shapes<'scan> {
    file: &'scan SourceFile,
    path: &'scan Path,
    depth: NestingDepth,
    violations: Vec<Violation<'scan>>,
}

impl<'scan> Shapes<'scan> {
    fn found(path: &'scan Path, span: Span, finding: Finding) -> Violation<'scan> {
        Violation::new(Site::new(path, LineNumber::from(span)), finding)
    }

    fn otherwise(&mut self, branch: Option<&Expr>) {
        match branch {
            None => {}
            Some(Expr::If(chained)) => self.visit_expr_if(chained),
            Some(other) => self.nested(other.span(), |this| this.visit_expr(other)),
        }
    }

    fn nested(&mut self, span: Span, visit: impl FnOnce(&mut Self)) {
        self.depth += NestingDepth::LEVEL;
        let first_past = NestingDepth::beyond(FnShape::NESTING);
        let deepest = (self.depth == first_past).then_some(span);
        self.violations.extend(
            deepest.map(|nested| Self::found(self.path, nested, Finding::DeepNesting { most: FnShape::NESTING })),
        );
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
        Macros::walk(self.file, invocation, |statement| self.visit_stmt(statement));
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_item_fn(self, function));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        Walk::enter(&function.attrs, || syn::visit::visit_impl_item_fn(self, function));
    }

    fn visit_signature(&mut self, signature: &'ast Signature) {
        let typed = signature.inputs.iter().filter(|argument| matches!(argument, FnArg::Typed(_))).count();
        let wide = FnShape::PARAMETERS.allows(Length::new(typed)).map_or(Some(signature.ident.span()), |_| None);
        self.violations.extend(
            wide.map(|span| Self::found(self.path, span, Finding::WideSignature { most: FnShape::PARAMETERS })),
        );
        syn::visit::visit_signature(self, signature);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        self.violations.extend(
            FnShape::binds_bool(local).map(|binding| Self::found(self.path, binding.span(), Finding::BooleanLocal)),
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
                .filter(|_| FnShape::FOLDS.iter().any(|fold| call.method == fold))
                .map(|_| Self::found(self.path, call.method.span(), Finding::TupleAccumulator)),
        );
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.visit_expr(&expr.cond);
        self.nested(expr.if_token.span, |this| {
            this.visit_block(&expr.then_branch);
        });
        self.otherwise(expr.else_branch.as_ref().map(|(_, branch)| &**branch));
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
