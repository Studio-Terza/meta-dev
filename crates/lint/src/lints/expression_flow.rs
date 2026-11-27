use std::collections::{BTreeMap, BTreeSet};

use invariants::Ignoring;
use proc_macro2::Ident;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprWhile, ImplItemConst,
    ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Signature, Stmt, Token,
    TraitItemFn,
};

use crate::expansion::Expansion;
use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::ignored::Ignored;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug, Default)]
pub(crate) struct ExpressionFlow {
    found: Vec<Found>,
    scopes: Vec<Locals>,
}

#[derive(Debug, Default)]
struct Locals {
    declared: BTreeMap<String, usize>,
    touched: BTreeSet<String>,
}

impl ExpressionFlow {
    const WRITING_MACROS: [&'static str; 2] = ["write", "writeln"];
    const LOOP_FIX: &'static str = "drive an iterator with combinators, const fns excepted";
    const ASSIGN_FIX: &'static str = "build the value in one expression or give its type an updater method";

    fn function(
        &mut self,
        attributes: &[Attribute],
        signature: &Signature,
        body: Option<&Block>,
        walk: impl FnOnce(&mut ExpressionFlow),
    ) {
        let scanned = Gate::of(attributes).entered().and(body).filter(|_| signature.constness.is_none());
        scanned.map_or_default(|_| {
            self.scopes.push(Locals::default());
            walk(self);
            let closed = self.scopes.pop().unwrap_or_default();
            let reassigned = closed.declared.into_iter().filter(|(name, _)| !closed.touched.contains(name));
            self.found.extend(reassigned.map(|(_, line)| {
                Found::at(
                    line,
                    "mutable local that is only reassigned",
                    "a mutable binding is borrowed as &mut or has methods called on it",
                )
            }));
        });
    }

    fn touched(&mut self, name: Option<&Ident>) {
        let scope = self.scopes.last_mut();
        scope.zip(name).map_or_default(|(locals, ident)| {
            locals.touched.insert(ident.to_string());
        });
    }

    fn assigned_local(place: &Expr) -> Option<&Ident> {
        match place {
            Expr::Field(field) => ExpressionFlow::assigned_local(&field.base),
            Expr::Index(index) => ExpressionFlow::assigned_local(&index.expr),
            Expr::Paren(paren) => ExpressionFlow::assigned_local(&paren.expr),
            Expr::Path(path) => path.path.get_ident().filter(|ident| *ident != "self"),
            _ => None,
        }
    }

    fn touched_by(expr: &Expr) -> Option<&Ident> {
        match expr {
            Expr::MethodCall(call) => ExpressionFlow::assigned_local(&call.receiver),
            Expr::Call(call) => ExpressionFlow::assigned_local(&call.func),
            Expr::Reference(reference) => {
                reference.mutability.and_then(|_| ExpressionFlow::assigned_local(&reference.expr))
            }
            _ => None,
        }
    }

    fn compound(op: BinOp) -> Option<BinOp> {
        Some(op).filter(|candidate| {
            matches!(
                candidate,
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

    fn written_to(invocation: &Macro) -> Option<Ident> {
        let writing = ExpressionFlow::WRITING_MACROS.iter().find(|name| invocation.path.is_ident(name));
        let arguments = writing.and_then(|_| {
            invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ignoring(Ignored::Unparsed)
        });
        arguments?.first().and_then(ExpressionFlow::assigned_local).cloned()
    }

    fn dropped(statement: &Stmt, last: Option<&Stmt>) -> Option<()> {
        let semicolon = match statement {
            Stmt::Expr(Expr::If(_) | Expr::Match(_), semicolon) => Some(semicolon),
            _ => None,
        }?;
        let inner = last.filter(|tail| std::ptr::eq(*tail, statement)).xor(Some(statement));
        semicolon.map(|_| ()).or(inner.map(|_| ()))
    }
}

impl<'ast> Visit<'ast> for ExpressionFlow {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        Gate::of(&item.attrs).entered().map_or_default(|()| visit::visit_item_impl(self, item));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        Gate::of(&item.attrs).entered().map_or_default(|()| visit::visit_item_trait(self, item));
    }

    fn visit_item_const(&mut self, _: &'ast ItemConst) {}

    fn visit_impl_item_const(&mut self, _: &'ast ImplItemConst) {}

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.function(&function.attrs, &function.sig, Some(&function.block), |flow| {
            visit::visit_item_fn(flow, function);
        });
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.function(&function.attrs, &function.sig, Some(&function.block), |flow| {
            visit::visit_impl_item_fn(flow, function);
        });
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.function(&function.attrs, &function.sig, function.default.as_ref(), |flow| {
            visit::visit_trait_item_fn(flow, function);
        });
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.touched(ExpressionFlow::written_to(invocation).as_ref());
        Expansion::of(invocation).walk(self);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let last = block.stmts.last();
        let dropped = block.stmts.iter().filter(|statement| ExpressionFlow::dropped(statement, last).is_some());
        let found: Vec<Found> = dropped
            .map(|statement| {
                Found::spanned(
                    statement.span(),
                    "if or match whose value is dropped",
                    "let it produce the value the function returns",
                )
            })
            .collect();
        self.found.extend(found);
        visit::visit_block(self, block);
    }

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.found.extend(expr.else_branch.is_none().then(|| {
            Found::spanned(
                expr.if_token.span,
                "if without else yields nothing",
                "use then_some, filter, or an else branch that produces the value",
            )
        }));
        visit::visit_expr_if(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.found.push(Found::spanned(expr.loop_token.span, "loop", ExpressionFlow::LOOP_FIX));
        visit::visit_expr_loop(self, expr);
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.found.push(Found::spanned(
            expr.for_token.span,
            "for loop",
            "drive the iterator with combinators, const fns excepted",
        ));
        visit::visit_expr_for_loop(self, expr);
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.found.push(Found::spanned(expr.while_token.span, "while loop", ExpressionFlow::LOOP_FIX));
        visit::visit_expr_while(self, expr);
    }

    fn visit_expr_assign(&mut self, expr: &'ast ExprAssign) {
        let local = ExpressionFlow::assigned_local(&expr.left);
        self.found.extend(
            local.map(|_| Found::spanned(expr.eq_token.span, "assignment to a local", ExpressionFlow::ASSIGN_FIX)),
        );
        visit::visit_expr_assign(self, expr);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        let local = ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op));
        self.found.extend(
            local.map(|op| Found::spanned(op.span(), "compound assignment to a local", ExpressionFlow::ASSIGN_FIX)),
        );
        visit::visit_expr_binary(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        let declared = match &local.pat {
            Pat::Ident(binding) => binding.mutability.map(|_| &binding.ident),
            _ => None,
        };
        let line = local.let_token.span.start().line;
        let scope = self.scopes.last_mut();
        scope.zip(declared).map_or_default(|(locals, name)| {
            locals.declared.insert(name.to_string(), line);
        });
        visit::visit_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.touched(ExpressionFlow::touched_by(expr));
        visit::visit_expr(self, expr);
    }
}

impl<'file> Style<'file> for ExpressionFlow {
    const RULE: Rule = Rule::ExpressionFlow;
    const READS: Reads = Reads::Shipped;

    fn new(_: &'file Source) -> ExpressionFlow {
        ExpressionFlow::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
