mod fn_body;

use crate::Lint;
use std::collections::BTreeMap;

use fn_body::FnBody;
use proc_macro2::Span;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

use syn::{
    BinOp, Block, Expr, ExprAssign, ExprBinary, ExprBreak, ExprContinue, ExprForLoop, ExprIf, ExprLoop, ExprReturn,
    ExprWhile, Ident, ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemMod, Local, Pat, PatIdent, Stmt, TraitItemFn,
};

use lint_core::LineNumber;
use lint_core::Name;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;

#[derive(Debug)]
pub struct ExpressionFlow;

impl Lint for ExpressionFlow {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "control flow is an expression; every branch yields the value the function returns",
            files.iter().filter(|file| file.is_shipped()).flat_map(Self::violations).collect(),
        )
    }
}

impl ExpressionFlow {
    fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut flows = Flows { path: file.path(), violations: Vec::new() };
        flows.visit_file(file.syntax());
        flows.violations
    }

    fn is_dropped(statement: &Stmt, last: bool) -> bool {
        match statement {
            Stmt::Expr(Expr::If(_) | Expr::Match(_), semicolon) => semicolon.is_some() || !last,
            _ => false,
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

    fn is_compound(op: BinOp) -> bool {
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
    }
}

struct Flows<'scan> {
    path: &'scan Path,
    violations: Vec<Violation>,
}

impl Flows<'_> {
    fn found(path: &Path, span: Span, message: &str) -> Violation {
        Violation::new(Site::new(path, LineNumber::from(span)), message)
    }

    fn report(&mut self, span: Span, message: &str) {
        self.violations.push(Self::found(self.path, span, message));
    }

    fn lonely_if(expr: &ExprIf) -> Option<Span> {
        expr.else_branch.is_none().then_some(expr.if_token.span)
    }

    fn visit_body(&mut self, body: &Block) {
        let mut locals = MutableLocals::default();
        locals.visit_block(body);
        self.violations
            .extend(locals.only_reassigned().into_iter().map(|span| {
                Self::found(
                    self.path,
                    span,
                    "mutable local that is only reassigned; a mutable binding is borrowed as &mut or has methods called on it",
                )
            }));
        self.visit_block(body);
    }
}

impl<'ast> Visit<'ast> for Flows<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        match Walk::outside_tests(&module.attrs) {
            Walk::Entered => syn::visit::visit_item_mod(self, module),
            Walk::Skipped => {}
        }
    }

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        FnBody::scanned_unless_const(&function.sig, Some(&function.block)).walked_by(self);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        FnBody::scanned_unless_const(&function.sig, Some(&function.block)).walked_by(self);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        FnBody::scanned_unless_const(&function.sig, function.default.as_ref()).walked_by(self);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let last = block.stmts.len().saturating_sub(1);
        self.violations.extend(
            block
                .stmts
                .iter()
                .enumerate()
                .filter(|(index, statement)| ExpressionFlow::is_dropped(statement, *index == last))
                .map(|(_, statement)| {
                    Self::found(
                        self.path,
                        statement.span(),
                        "if or match whose value is dropped; let it produce the value the function returns",
                    )
                }),
        );
        syn::visit::visit_block(self, block);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.violations.extend(Self::lonely_if(expr).map(|span| {
            Self::found(
                self.path,
                span,
                "if without else yields nothing; use then_some, filter, or an else branch that produces the value",
            )
        }));
        syn::visit::visit_expr_if(self, expr);
    }

    fn visit_expr_return(&mut self, expr: &'ast ExprReturn) {
        self.report(
            expr.return_token.span,
            "early return; make the value the tail expression or exit through ? on a Result",
        );
        syn::visit::visit_expr_return(self, expr);
    }

    fn visit_expr_break(&mut self, expr: &'ast ExprBreak) {
        self.report(expr.break_token.span, "break; fold with ControlFlow or find the element with an iterator");
        syn::visit::visit_expr_break(self, expr);
    }

    fn visit_expr_continue(&mut self, expr: &'ast ExprContinue) {
        self.report(expr.continue_token.span, "continue; filter the iterator instead");
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.report(expr.loop_token.span, "loop; drive an iterator with combinators, const fns excepted");
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.report(expr.for_token.span, "for loop; drive the iterator with combinators, const fns excepted");
        syn::visit::visit_expr_for_loop(self, expr);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.report(expr.while_token.span, "while loop; drive an iterator with combinators, const fns excepted");
        syn::visit::visit_expr_while(self, expr);
    }

    fn visit_expr_assign(&mut self, expr: &'ast ExprAssign) {
        self.violations.extend(ExpressionFlow::assigned_local(&expr.left).map(|_| {
            Self::found(
                self.path,
                expr.eq_token.span,
                "assignment to a local; build the value in one expression or give its type an updater method",
            )
        }));
        syn::visit::visit_expr_assign(self, expr);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        self.violations.extend(
            ExpressionFlow::assigned_local(&expr.left)
                .filter(|_| ExpressionFlow::is_compound(expr.op))
                .map(|_| {
                    Self::found(
                        self.path,
                        expr.op.span(),
                        "compound assignment to a local; build the value in one expression or give its type an updater method",
                    )
                }),
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
    fn declared_mutable(local: &Local) -> Option<&PatIdent> {
        match &local.pat {
            Pat::Ident(binding) if binding.mutability.is_some() => Some(binding),
            _ => None,
        }
    }

    fn touched_by(expr: &Expr) -> Option<&Ident> {
        match expr {
            Expr::MethodCall(call) => ExpressionFlow::assigned_local(&call.receiver),
            Expr::Reference(reference) if reference.mutability.is_some() => {
                ExpressionFlow::assigned_local(&reference.expr)
            }
            _ => None,
        }
    }

    fn only_reassigned(&self) -> Vec<Span> {
        self.declared.iter().filter(|(name, _)| !self.touched.contains(*name)).map(|(_, span)| *span).collect()
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
}
