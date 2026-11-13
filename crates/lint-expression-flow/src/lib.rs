use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Descend, Finds, Kind, Sweep, Watch};
use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use std::path::Path;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use syn::{
    Attribute, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprWhile, Ident,
    ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, PatIdent, Signature,
    Stmt, Token, TraitItemFn,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_core::Walk;
use lint_layout::LineNumber;
use lint_layout::Name;

#[derive(Debug)]
pub struct ExpressionFlow;

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
}

impl SweptLint for ExpressionFlow {
    const HEADLINE: Headline =
        Headline::new("control flow is an expression; every branch yields the value the function returns");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Flows { file, path: file.head().path(), violations: Vec::new() })
    }
}

struct Flows<'scan> {
    file: &'scan SourceFile,
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
}

impl Flows<'_> {
    fn report(&mut self, spans: impl IntoIterator<Item = Span>, message: Message) {
        let path = self.path;
        self.violations
            .extend(spans.into_iter().map(|span| Violation::new(Site::new(path, LineNumber::from(span)), message)));
    }

    fn function(&mut self, attributes: &[Attribute], signature: &Signature, body: Option<&Block>) -> Descend {
        let scanned = Descend::from(Walk::outside_tests(attributes))
            .entered()
            .and(body)
            .filter(|_| signature.constness.is_none());
        scanned.map_or(Descend::Over, |block| self.body(block))
    }

    fn lonely_if(expr: &ExprIf) -> Option<Span> {
        expr.else_branch.is_none().then_some(expr.if_token.span)
    }

    fn body(&mut self, body: &Block) -> Descend {
        let mut locals = MutableLocals { declared: BTreeMap::new(), touched: Vec::new() };
        Sweep::through_block(self.file, body, &mut locals);
        self.report(
            locals.only_reassigned(),
            Message::new(
                Problem::new("mutable local that is only reassigned"),
                Fix::new("a mutable binding is borrowed as &mut or has methods called on it"),
            ),
        );
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Flows<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemConst,
            Kind::ImplItemConst,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::TraitItemFn,
            Kind::Block,
            Kind::ExprIf,
            Kind::ExprLoop,
            Kind::ExprForLoop,
            Kind::ExprWhile,
            Kind::ExprAssign,
            Kind::ExprBinary,
        ]
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_item_trait(&mut self, item: &'ast ItemTrait) -> Descend {
        Descend::from(Walk::outside_tests(&item.attrs))
    }

    fn on_macro(&mut self, _: &'ast Macro) -> Descend {
        Descend::Expand
    }

    fn on_item_const(&mut self, _: &'ast ItemConst) -> Descend {
        Descend::Over
    }

    fn on_impl_item_const(&mut self, _: &'ast ImplItemConst) -> Descend {
        Descend::Over
    }

    fn on_item_fn(&mut self, function: &'ast ItemFn) -> Descend {
        self.function(&function.attrs, &function.sig, Some(&function.block))
    }

    fn on_impl_item_fn(&mut self, function: &'ast ImplItemFn) -> Descend {
        self.function(&function.attrs, &function.sig, Some(&function.block))
    }

    fn on_trait_item_fn(&mut self, function: &'ast TraitItemFn) -> Descend {
        self.function(&function.attrs, &function.sig, function.default.as_ref())
    }

    fn on_block(&mut self, block: &'ast Block) -> Descend {
        let last = block.stmts.last();
        self.report(
            block.stmts.iter().filter_map(|statement| ExpressionFlow::dropped(statement, last)).map(Spanned::span),
            Message::new(
                Problem::new("if or match whose value is dropped"),
                Fix::new("let it produce the value the function returns"),
            ),
        );
        Descend::Into
    }

    fn on_expr_if(&mut self, expr: &'ast ExprIf) -> Descend {
        self.report(
            Self::lonely_if(expr),
            Message::new(
                Problem::new("if without else yields nothing"),
                Fix::new("use then_some, filter, or an else branch that produces the value"),
            ),
        );
        Descend::Into
    }

    fn on_expr_loop(&mut self, expr: &'ast ExprLoop) -> Descend {
        self.report(
            [expr.loop_token.span],
            Message::new(Problem::new("loop"), Fix::new("drive an iterator with combinators, const fns excepted")),
        );
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop) -> Descend {
        self.report(
            [expr.for_token.span],
            Message::new(Problem::new("for loop"), Fix::new("drive the iterator with combinators, const fns excepted")),
        );
        Descend::Into
    }

    fn on_expr_while(&mut self, expr: &'ast ExprWhile) -> Descend {
        self.report(
            [expr.while_token.span],
            Message::new(
                Problem::new("while loop"),
                Fix::new("drive an iterator with combinators, const fns excepted"),
            ),
        );
        Descend::Into
    }

    fn on_expr_assign(&mut self, expr: &'ast ExprAssign) -> Descend {
        self.report(
            ExpressionFlow::assigned_local(&expr.left).map(|_| expr.eq_token.span),
            Message::new(
                Problem::new("assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        );
        Descend::Into
    }

    fn on_expr_binary(&mut self, expr: &'ast ExprBinary) -> Descend {
        self.report(
            ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op)).map(|_| expr.op.span()),
            Message::new(
                Problem::new("compound assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        );
        Descend::Into
    }
}

struct MutableLocals {
    declared: BTreeMap<Name, Span>,
    touched: Vec<Name>,
}

impl MutableLocals {
    const WRITING_MACROS: [&'static str; 2] = ["write", "writeln"];

    fn declared_mutable(local: &Local) -> Option<&PatIdent> {
        match &local.pat {
            Pat::Ident(binding) => Some(binding).filter(|candidate| candidate.mutability.is_some()),
            _ => None,
        }
    }

    fn touched_by(expr: &Expr) -> Option<&Ident> {
        match expr {
            Expr::MethodCall(call) => ExpressionFlow::assigned_local(&call.receiver),
            Expr::Call(call) => ExpressionFlow::assigned_local(&call.func),
            Expr::Reference(reference) => Some(reference)
                .filter(|candidate| candidate.mutability.is_some())
                .and_then(|mutable| ExpressionFlow::assigned_local(&mutable.expr)),
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

impl<'ast> Watch<'ast> for MutableLocals {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Local, Kind::Expr, Kind::Macro]
    }

    fn on_local(&mut self, local: &'ast Local) -> Descend {
        self.declared
            .extend(Self::declared_mutable(local).map(|binding| (Name::from(&binding.ident), local.let_token.span)));
        Descend::Into
    }

    fn on_expr(&mut self, expr: &'ast Expr) -> Descend {
        self.touched.extend(Self::touched_by(expr).map(Name::from));
        Descend::Into
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        self.touched.extend(Self::written_to(invocation));
        Descend::Expand
    }
}

impl<'file> Finds<'file> for Flows<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}
