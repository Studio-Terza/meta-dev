mod fn_body;

use lint_core::{Headline, Lint};
use lint_file::FileLint;
use std::collections::{BTreeMap, BTreeSet};

use fn_body::FnBody;
use proc_macro2::Span;
use std::path::Path;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    Attribute, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprWhile, Ident,
    ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, PatIdent, Signature,
    Stmt, Token, TraitItemFn,
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
use lint_layout::Name;

#[derive(Debug)]
pub struct ExpressionFlow;

impl Lint for ExpressionFlow {
    fn report(files: &[SourceFile]) -> Report<'_> {
        Report::new(
            Headline::new("control flow is an expression; every branch yields the value the function returns"),
            Self::in_shipped(files),
        )
    }
}

impl ExpressionFlow {
    fn dropped<'block>(statement: &'block Stmt, last: Option<&Stmt>) -> Option<&'block Stmt> {
        let inner = last.filter(|tail| std::ptr::eq(*tail, statement)).map_or(Some(statement), |_| None);
        Self::branching(statement).and_then(|semicolon| semicolon.as_ref().map(|_| statement).or(inner))
    }

    fn branching(statement: &Stmt) -> Option<&Option<Token![;]>> {
        match statement {
            Stmt::Expr(Expr::If(_) | Expr::Match(_), semicolon) => Some(semicolon),
            _ => None,
        }
    }

    fn assigned_local(place: &Expr) -> Option<&Ident> {
        match place {
            Expr::Path(path) => path.path.get_ident().filter(|ident| *ident != "self"),
            Expr::Field(field) => Self::assigned_local(&field.base),
            Expr::Index(index) => Self::assigned_local(&index.expr),
            Expr::Paren(paren) => Self::assigned_local(&paren.expr),
            _ => None,
        }
    }

    fn compound(op: BinOp) -> Option<BinOp> {
        Some(op).filter(|op| {
            matches!(
                op,
                BinOp::AddAssign(_)
                    | BinOp::SubAssign(_)
                    | BinOp::MulAssign(_)
                    | BinOp::DivAssign(_)
                    | BinOp::RemAssign(_)
                    | BinOp::BitXorAssign(_)
                    | BinOp::BitAndAssign(_)
                    | BinOp::BitOrAssign(_)
                    | BinOp::ShlAssign(_)
                    | BinOp::ShrAssign(_)
            )
        })
    }
}

impl FileLint for ExpressionFlow {
    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let mut flows = Flows { path: file.path(), violations: Vec::new() };
        flows.visit_file(file.syntax());
        flows.violations
    }
}

struct Flows<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl Flows<'_> {
    fn report(&mut self, spans: impl IntoIterator<Item = Span>, message: Message) {
        let path = self.path;
        self.violations
            .extend(spans.into_iter().map(|span| Violation::new(Site::new(path, LineNumber::from(span)), message)));
    }

    fn function(&mut self, attributes: &[Attribute], signature: &Signature, body: Option<&Block>) {
        Walk::enter(attributes, || FnBody::scanned_unless_const(signature, body).walked_by(self));
    }

    fn lonely_if(expr: &ExprIf) -> Option<Span> {
        expr.else_branch.is_none().then_some(expr.if_token.span)
    }

    fn visit_body(&mut self, body: &Block) {
        let mut locals = MutableLocals::default();
        locals.visit_block(body);
        self.report(
            locals.only_reassigned(),
            Message::new(
                Problem::new("mutable local that is only reassigned"),
                Fix::new("a mutable binding is borrowed as &mut or has methods called on it"),
            ),
        );
        self.visit_block(body);
    }
}

impl<'ast> Visit<'ast> for Flows<'_> {
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

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.function(&function.attrs, &function.sig, Some(&function.block));
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.function(&function.attrs, &function.sig, Some(&function.block));
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.function(&function.attrs, &function.sig, function.default.as_ref());
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let last = block.stmts.last();
        self.report(
            block.stmts.iter().filter_map(|statement| ExpressionFlow::dropped(statement, last)).map(Spanned::span),
            Message::new(
                Problem::new("if or match whose value is dropped"),
                Fix::new("let it produce the value the function returns"),
            ),
        );
        syn::visit::visit_block(self, block);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.report(
            Self::lonely_if(expr),
            Message::new(
                Problem::new("if without else yields nothing"),
                Fix::new(
                    "give the decision a named outcome, an enum a pure router matches, so each case produces a value",
                ),
            ),
        );
        syn::visit::visit_expr_if(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.report(
            [expr.loop_token.span],
            Message::new(Problem::new("loop"), Fix::new("drive an iterator with combinators, const fns excepted")),
        );
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.report(
            [expr.for_token.span],
            Message::new(Problem::new("for loop"), Fix::new("drive the iterator with combinators, const fns excepted")),
        );
        syn::visit::visit_expr_for_loop(self, expr);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.report(
            [expr.while_token.span],
            Message::new(
                Problem::new("while loop"),
                Fix::new("drive an iterator with combinators, const fns excepted"),
            ),
        );
        syn::visit::visit_expr_while(self, expr);
    }

    fn visit_expr_assign(&mut self, expr: &'ast ExprAssign) {
        self.report(
            ExpressionFlow::assigned_local(&expr.left).map(|_| expr.eq_token.span),
            Message::new(
                Problem::new("assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        );
        syn::visit::visit_expr_assign(self, expr);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        self.report(
            ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op)).map(|_| expr.op.span()),
            Message::new(
                Problem::new("compound assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        );
        syn::visit::visit_expr_binary(self, expr);
    }
}

#[derive(Default)]
struct MutableLocals {
    declared: BTreeMap<Name, Span>,
    touched: Vec<Name>,
}

impl MutableLocals {
    const WRITING_MACROS: [&'static str; 2] = ["write", "writeln"];

    fn declared_mutable(local: &Local) -> Option<&PatIdent> {
        match &local.pat {
            Pat::Ident(binding) => Some(binding).filter(|binding| binding.mutability.is_some()),
            _ => None,
        }
    }

    fn touched_by(expr: &Expr) -> Option<&Ident> {
        match expr {
            Expr::MethodCall(call) => ExpressionFlow::assigned_local(&call.receiver),
            Expr::Call(call) => ExpressionFlow::assigned_local(&call.func),
            Expr::Reference(reference) => Some(reference)
                .filter(|reference| reference.mutability.is_some())
                .and_then(|reference| ExpressionFlow::assigned_local(&reference.expr)),
            _ => None,
        }
    }

    fn written_to(invocation: &Macro) -> Option<Name> {
        let arguments = invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok()?;
        Self::WRITING_MACROS
            .iter()
            .any(|name| invocation.path.is_ident(name))
            .then_some(arguments)?
            .first()
            .and_then(ExpressionFlow::assigned_local)
            .map(Name::from)
    }

    fn only_reassigned(&self) -> Vec<Span> {
        let touched: BTreeSet<&Name> = self.touched.iter().collect();
        let declared: BTreeSet<&Name> = self.declared.keys().collect();
        declared.difference(&touched).filter_map(|name| self.declared.get(*name)).copied().collect()
    }
}

impl<'ast> Visit<'ast> for MutableLocals {
    fn visit_local(&mut self, local: &'ast Local) {
        self.declared
            .extend(Self::declared_mutable(local).map(|binding| (Name::from(&binding.ident), local.let_token.span)));
        syn::visit::visit_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.touched.extend(Self::touched_by(expr).map(Name::from));
        syn::visit::visit_expr(self, expr);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        Macros::walk(invocation, |statement| self.visit_stmt(statement));
        self.touched.extend(Self::written_to(invocation));
    }
}
