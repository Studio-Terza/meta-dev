use std::collections::{BTreeMap, BTreeSet};

use invariants::Ignoring;
use proc_macro2::Ident;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprWhile, ImplItemConst,
    ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Signature, Stmt, Token,
    TraitItemFn,
};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
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

    fn opened(&mut self, attributes: &[Attribute], signature: &Signature, body: Option<&Block>) -> Descent {
        let scanned = Gate::of(attributes).entered().and(body).filter(|_| signature.constness.is_none());
        self.scopes.extend(scanned.map(|_| Locals::default()));
        Descent::from(scanned.map(|_| ()))
    }

    fn closed(&mut self) {
        let closed = self.scopes.pop().unwrap_or_default();
        let reassigned = closed.declared.into_iter().filter(|(name, _)| !closed.touched.contains(name));
        self.found.extend(reassigned.map(|(_, line)| {
            Found::at(
                line,
                "mutable local that is only reassigned",
                "a mutable binding is borrowed as &mut or has methods called on it",
            )
        }));
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

impl Hooks for ExpressionFlow {
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        Descent::from(Gate::of(&module.attrs).entered())
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::of(&item.attrs).entered())
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        Descent::from(Gate::of(&item.attrs).entered())
    }

    fn item_const(&mut self, _: &ItemConst) -> Descent {
        Descent::Over
    }

    fn impl_item_const(&mut self, _: &ImplItemConst) -> Descent {
        Descent::Over
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        self.opened(&function.attrs, &function.sig, Some(&function.block))
    }

    fn left_item_fn(&mut self, _: &ItemFn) {
        self.closed();
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        self.opened(&function.attrs, &function.sig, Some(&function.block))
    }

    fn left_impl_item_fn(&mut self, _: &ImplItemFn) {
        self.closed();
    }

    fn trait_item_fn(&mut self, function: &TraitItemFn) -> Descent {
        self.opened(&function.attrs, &function.sig, function.default.as_ref())
    }

    fn left_trait_item_fn(&mut self, _: &TraitItemFn) {
        self.closed();
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        self.touched(ExpressionFlow::written_to(invocation).as_ref());
        Expanded::Statements
    }

    fn block(&mut self, block: &Block) -> Descent {
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
        Descent::Into
    }

    fn expr_if(&mut self, expr: &ExprIf) -> Descent {
        self.found.extend(expr.else_branch.is_none().then(|| {
            Found::spanned(
                expr.if_token.span,
                "if without else yields nothing",
                "use then_some, filter, or an else branch that produces the value",
            )
        }));
        Descent::Into
    }

    fn expr_loop(&mut self, expr: &ExprLoop) -> Descent {
        self.found.push(Found::spanned(expr.loop_token.span, "loop", ExpressionFlow::LOOP_FIX));
        Descent::Into
    }

    fn expr_for_loop(&mut self, expr: &ExprForLoop) -> Descent {
        self.found.push(Found::spanned(
            expr.for_token.span,
            "for loop",
            "drive the iterator with combinators, const fns excepted",
        ));
        Descent::Into
    }

    fn expr_while(&mut self, expr: &ExprWhile) -> Descent {
        self.found.push(Found::spanned(expr.while_token.span, "while loop", ExpressionFlow::LOOP_FIX));
        Descent::Into
    }

    fn expr_assign(&mut self, expr: &ExprAssign) -> Descent {
        let local = ExpressionFlow::assigned_local(&expr.left);
        self.found.extend(
            local.map(|_| Found::spanned(expr.eq_token.span, "assignment to a local", ExpressionFlow::ASSIGN_FIX)),
        );
        Descent::Into
    }

    fn expr_binary(&mut self, expr: &ExprBinary) -> Descent {
        let local = ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op));
        self.found.extend(
            local.map(|op| Found::spanned(op.span(), "compound assignment to a local", ExpressionFlow::ASSIGN_FIX)),
        );
        Descent::Into
    }

    fn local(&mut self, local: &Local) -> Descent {
        let declared = match &local.pat {
            Pat::Ident(binding) => binding.mutability.map(|_| &binding.ident),
            _ => None,
        };
        let line = local.let_token.span.start().line;
        let scope = self.scopes.last_mut();
        scope.zip(declared).map_or_default(|(locals, name)| {
            locals.declared.insert(name.to_string(), line);
        });
        Descent::Into
    }

    fn expr(&mut self, expr: &Expr) -> Descent {
        self.touched(ExpressionFlow::touched_by(expr));
        Descent::Into
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
