use lint_core::Headline;
use lint_core::{Drawn, Fold, Folding, Visit};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Attaching, Attachment, Descend, Facts, Finds, Kind, Scoped};
use std::collections::{BTreeMap, BTreeSet};

use std::path::Path;
use syntax::Punctuated;
use syntax::Span;
use syntax::Spanned;
use syntax::Spine;

use syntax::{
    Attribute, BinOp, Block, Expr, ExprAssign, ExprBinary, ExprForLoop, ExprIf, ExprLoop, ExprWhile, Ident,
    ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, PatIdent, Signature,
    Stmt, Token, TraitItemFn,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Syntax;
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
        let root = Spine::from(place).past(|node| matches!(node, Expr::Field(_) | Expr::Index(_) | Expr::Paren(_)))?;
        Syntax::expr_path(root.node()).and_then(|path| path.path.get_ident()).filter(|ident| *ident != "self")
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
        Box::new(Scoped::new(Folding::from(Flows { path: AsRef::<Path>::as_ref(file.head()) })))
    }
}

#[derive(Debug)]
struct Flows<'scan> {
    path: &'scan Path,
}

#[derive(Debug, Default)]
struct Flowed<'scan> {
    violations: Vec<Violation<'scan>>,
    opening: Vec<Attachment<MutableLocals>>,
    held: Vec<Vec<Violation<'scan>>>,
}

impl<'scan> Flows<'scan> {
    fn flagged(&self, spans: impl IntoIterator<Item = Span>, message: Message) -> Vec<Violation<'scan>> {
        let path = self.path;
        spans.into_iter().map(|span| Violation::new(Site::new(path, LineNumber::from(span)), message)).collect()
    }

    fn report(
        &self,
        found: Flowed<'scan>,
        spans: impl IntoIterator<Item = Span>,
        message: Message,
    ) -> Visit<Flowed<'scan>> {
        Visit::descending(found.flagged(self.flagged(spans, message)))
    }

    fn function(
        found: Flowed<'scan>,
        attributes: &[Attribute],
        signature: &Signature,
        body: Option<&Block>,
    ) -> Visit<Flowed<'scan>> {
        let scanned = Descend::from(Walk::outside_tests(attributes))
            .entered()
            .and(body)
            .filter(|_| signature.constness.is_none());
        found.opened(scanned.map(|block| Attachment::reaching(block, MutableLocals)))
    }

    fn lonely_if(expr: &ExprIf) -> Option<Span> {
        expr.else_branch.is_none().then_some(expr.if_token.span)
    }
}

impl<'scan> Flowed<'scan> {
    fn flagged(self, more: Vec<Violation<'scan>>) -> Flowed<'scan> {
        Flowed { violations: self.violations.into_iter().chain(more).collect(), ..self }
    }

    fn opened(self, opened: Option<Attachment<MutableLocals>>) -> Visit<Flowed<'scan>> {
        match opened {
            Some(attachment) => Visit::descending(Flowed {
                violations: Vec::new(),
                opening: self.opening.into_iter().chain([attachment]).collect(),
                held: self.held.into_iter().chain([self.violations]).collect(),
            }),
            None => Visit::new(self, Descend::Over),
        }
    }
}

impl<'scan> From<Flowed<'scan>> for Vec<Violation<'scan>> {
    fn from(flowed: Flowed<'scan>) -> Vec<Violation<'scan>> {
        flowed.violations
    }
}

impl<'scan> Fold for Flows<'scan> {
    type Found = Flowed<'scan>;

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

    fn on_item_mod(&self, found: Flowed<'scan>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(&self, found: Flowed<'scan>, _: &ItemImpl, facts: &Facts<'_, ItemImpl>) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_trait(&self, found: Flowed<'scan>, _: &ItemTrait, facts: &Facts<'_, ItemTrait>) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_macro(&self, found: Flowed<'scan>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_item_const(&self, found: Flowed<'scan>, _: &ItemConst, _: &Facts<'_, ItemConst>) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::Over)
    }

    fn on_impl_item_const(
        &self,
        found: Flowed<'scan>,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
    ) -> Visit<Flowed<'scan>> {
        Visit::new(found, Descend::Over)
    }

    fn on_item_fn(&self, found: Flowed<'scan>, function: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Flowed<'scan>> {
        Self::function(found, &function.attrs, &function.sig, Some(&function.block))
    }

    fn on_impl_item_fn(
        &self,
        found: Flowed<'scan>,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Flowed<'scan>> {
        Self::function(found, &function.attrs, &function.sig, Some(&function.block))
    }

    fn on_trait_item_fn(
        &self,
        found: Flowed<'scan>,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Flowed<'scan>> {
        Self::function(found, &function.attrs, &function.sig, function.default.as_ref())
    }

    fn on_block(&self, found: Flowed<'scan>, block: &Block, _: &Facts<'_, Block>) -> Visit<Flowed<'scan>> {
        let last = block.stmts.last();
        self.report(
            found,
            block.stmts.iter().filter_map(|statement| ExpressionFlow::dropped(statement, last)).map(Spanned::span),
            Message::new(
                Problem::new("if or match whose value is dropped"),
                Fix::new("let it produce the value the function returns"),
            ),
        )
    }

    fn on_expr_if(&self, found: Flowed<'scan>, expr: &ExprIf, _: &Facts<'_, ExprIf>) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            Self::lonely_if(expr),
            Message::new(
                Problem::new("if without else yields nothing"),
                Fix::new("use then_some, filter, or an else branch that produces the value"),
            ),
        )
    }

    fn on_expr_loop(&self, found: Flowed<'scan>, expr: &ExprLoop, _: &Facts<'_, ExprLoop>) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            [expr.loop_token.span],
            Message::new(Problem::new("loop"), Fix::new("drive an iterator with combinators, const fns excepted")),
        )
    }

    fn on_expr_for_loop(
        &self,
        found: Flowed<'scan>,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            [expr.for_token.span],
            Message::new(Problem::new("for loop"), Fix::new("drive the iterator with combinators, const fns excepted")),
        )
    }

    fn on_expr_while(&self, found: Flowed<'scan>, expr: &ExprWhile, _: &Facts<'_, ExprWhile>) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            [expr.while_token.span],
            Message::new(
                Problem::new("while loop"),
                Fix::new("drive an iterator with combinators, const fns excepted"),
            ),
        )
    }

    fn on_expr_assign(
        &self,
        found: Flowed<'scan>,
        expr: &ExprAssign,
        _: &Facts<'_, ExprAssign>,
    ) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            ExpressionFlow::assigned_local(&expr.left).map(|_| expr.eq_token.span),
            Message::new(
                Problem::new("assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        )
    }

    fn on_expr_binary(
        &self,
        found: Flowed<'scan>,
        expr: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
    ) -> Visit<Flowed<'scan>> {
        self.report(
            found,
            ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op)).map(|_| expr.op.span()),
            Message::new(
                Problem::new("compound assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        )
    }
}

#[derive(Debug)]
struct MutableLocals;

#[derive(Debug, Default)]
struct Locals {
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
        let writing = Self::WRITING_MACROS.iter().find(|name| invocation.path.is_ident(name));
        let arguments =
            writing.and_then(|_| invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok());
        arguments?.first().and_then(ExpressionFlow::assigned_local).map(Name::from)
    }
}

impl Locals {
    fn only_reassigned(&self) -> Vec<Span> {
        let touched: BTreeSet<&Name> = self.touched.iter().collect();
        let declared: BTreeSet<&Name> = self.declared.keys().collect();
        declared.difference(&touched).filter_map(|name| self.declared.get(*name)).copied().collect()
    }

    fn touching(self, touched: Option<Name>) -> Locals {
        Locals { touched: self.touched.into_iter().chain(touched).collect(), ..self }
    }
}

impl Fold for MutableLocals {
    type Found = Locals;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Local, Kind::Expr, Kind::Macro]
    }

    fn on_local(&self, found: Locals, local: &Local, _: &Facts<'_, Local>) -> Visit<Locals> {
        let declared = Self::declared_mutable(local).map(|binding| (Name::from(&binding.ident), local.let_token.span));
        Visit::descending(Locals { declared: found.declared.into_iter().chain(declared).collect(), ..found })
    }

    fn on_expr(&self, found: Locals, expr: &Expr, _: &Facts<'_, Expr>) -> Visit<Locals> {
        Visit::descending(found.touching(Self::touched_by(expr).map(Name::from)))
    }

    fn on_macro(&self, found: Locals, invocation: &Macro, _: &Facts<'_, Macro>) -> Visit<Locals> {
        Visit::new(found.touching(Self::written_to(invocation)), Descend::Expand)
    }
}

impl<'scan> Attaching<MutableLocals> for Flows<'scan> {
    const ATTACHED: &'static [Kind] = &[Kind::Block, Kind::Local, Kind::Expr, Kind::Macro];

    fn attaching(&self, found: Flowed<'scan>) -> Drawn<Flowed<'scan>, Vec<Attachment<MutableLocals>>> {
        Drawn::new(Flowed { opening: Vec::new(), ..found }, found.opening)
    }

    fn detached(&self, found: Flowed<'scan>, locals: Locals) -> Flowed<'scan> {
        let mut held = found.held;
        let outer = held.pop().unwrap_or_default();
        let reassigned = self.flagged(
            locals.only_reassigned(),
            Message::new(
                Problem::new("mutable local that is only reassigned"),
                Fix::new("a mutable binding is borrowed as &mut or has methods called on it"),
            ),
        );
        let violations = outer.into_iter().chain(reassigned).chain(found.violations).collect();
        Flowed { violations, held, ..found }
    }
}
