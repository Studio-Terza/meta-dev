mod ignored;

use algebra::{Monoid, Semigroup};
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Finds, Kind, Scope};
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

use crate::ignored::Ignored;
use invariants::Ignoring;
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

    #[must_use]
    pub fn lonely_if(expr: &ExprIf) -> Option<Span> {
        expr.else_branch.is_none().then_some(expr.if_token.span)
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

    fn watcher(file: &SourceFile) -> impl Finds<'_> {
        Contributing::new(Flows { path: AsRef::<Path>::as_ref(file.head()) })
    }
}

#[derive(Debug)]
struct Flows<'scan> {
    path: &'scan Path,
}

#[derive(Debug)]
struct Declared {
    name: Name,
    span: Span,
}

#[derive(Debug, Default)]
struct Flowed<'scan> {
    found: Vec<Violation<'scan>>,
    declared: Vec<Declared>,
    touched: Vec<Name>,
}

impl Semigroup for Flowed<'_> {
    fn combine(self, other: Self) -> Self {
        Flowed {
            found: self.found.combine(other.found),
            declared: self.declared.combine(other.declared),
            touched: self.touched.combine(other.touched),
        }
    }
}

impl Monoid for Flowed<'_> {}

impl Flowed<'_> {
    fn only_reassigned(&self) -> Vec<Span> {
        let declared: BTreeMap<&Name, Span> = self.declared.iter().map(|local| (&local.name, local.span)).collect();
        let touched: BTreeSet<&Name> = self.touched.iter().collect();
        declared
            .into_iter()
            .filter(|(name, _)| touched.contains(name).then_some(()).is_none())
            .map(|(_, span)| span)
            .collect()
    }
}

impl<'scan> Flows<'scan> {
    fn reported(&self, spans: impl IntoIterator<Item = Span>, message: Message) -> Contributed<(), Flowed<'scan>> {
        let path = self.path;
        let found = spans.into_iter().map(|span| Violation::new(Site::new(path, LineNumber::from(span)), message));
        Contributed::parted(Flowed { found: found.collect(), ..Flowed::default() })
    }

    fn scanned<'body>(
        attributes: &[Attribute],
        signature: &Signature,
        body: Option<&'body Block>,
    ) -> Option<&'body Block> {
        Descend::from(Walk::outside_tests(attributes)).entered().and(body).filter(|_| signature.constness.is_none())
    }

    fn function(
        attributes: &[Attribute],
        signature: &Signature,
        body: Option<&Block>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Self::scanned(attributes, signature, body).map_or(Descend::Over, |_| Descend::Into))
    }

    fn closed(
        &self,
        attributes: &[Attribute],
        signature: &Signature,
        body: Option<&Block>,
        inner: Contribution<(), Flowed<'scan>>,
    ) -> Flowed<'scan> {
        let path = self.path;
        inner.concluded(|(), flowed| {
            let reassigned =
                Self::scanned(attributes, signature, body).map_or_else(Vec::new, |_| flowed.only_reassigned());
            let message = Message::new(
                Problem::new("mutable local that is only reassigned"),
                Fix::new("a mutable binding is borrowed as &mut or has methods called on it"),
            );
            let found =
                reassigned.into_iter().map(|span| Violation::new(Site::new(path, LineNumber::from(span)), message));
            Flowed { found: flowed.found.into_iter().chain(found).collect(), ..flowed }
        })
    }
}

impl<'scan> Contribute for Flows<'scan> {
    type Open = ();
    type Part = Flowed<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemConst,
            Kind::ImplItemConst,
            Kind::Block,
            Kind::ExprIf,
            Kind::ExprLoop,
            Kind::ExprForLoop,
            Kind::ExprWhile,
            Kind::ExprAssign,
            Kind::ExprBinary,
            Kind::Local,
            Kind::Expr,
        ]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemFn), Scope::new(Kind::ImplItemFn), Scope::new(Kind::TraitItemFn)] }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_trait(
        &self,
        _: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_macro(
        &self,
        invocation: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        let touched = MutableLocals::written_to(invocation).into_iter().collect();
        Contributed::new((), Flowed { touched, ..Flowed::default() }, Descend::Expand)
    }

    fn on_item_const(
        &self,
        _: &ItemConst,
        _: &Facts<'_, ItemConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Descend::Over)
    }

    fn on_impl_item_const(
        &self,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Contributed::descending(Descend::Over)
    }

    fn on_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Self::function(&function.attrs, &function.sig, Some(&function.block))
    }

    fn off_item_fn(
        &self,
        function: &ItemFn,
        _: &Facts<'_, ItemFn>,
        inner: Contribution<(), Flowed<'scan>>,
    ) -> Flowed<'scan> {
        self.closed(&function.attrs, &function.sig, Some(&function.block), inner)
    }

    fn on_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Self::function(&function.attrs, &function.sig, Some(&function.block))
    }

    fn off_impl_item_fn(
        &self,
        function: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<(), Flowed<'scan>>,
    ) -> Flowed<'scan> {
        self.closed(&function.attrs, &function.sig, Some(&function.block), inner)
    }

    fn on_trait_item_fn(
        &self,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        Self::function(&function.attrs, &function.sig, function.default.as_ref())
    }

    fn off_trait_item_fn(
        &self,
        function: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
        inner: Contribution<(), Flowed<'scan>>,
    ) -> Flowed<'scan> {
        self.closed(&function.attrs, &function.sig, function.default.as_ref(), inner)
    }

    fn on_block(
        &self,
        block: &Block,
        _: &Facts<'_, Block>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        let last = block.stmts.last();
        self.reported(
            block.stmts.iter().filter_map(|statement| ExpressionFlow::dropped(statement, last)).map(Spanned::span),
            Message::new(
                Problem::new("if or match whose value is dropped"),
                Fix::new("let it produce the value the function returns"),
            ),
        )
    }

    fn on_expr_if(
        &self,
        expr: &ExprIf,
        _: &Facts<'_, ExprIf>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            ExpressionFlow::lonely_if(expr),
            Message::new(
                Problem::new("if without else yields nothing"),
                Fix::new("use then_some, filter, or an else branch that produces the value"),
            ),
        )
    }

    fn on_expr_loop(
        &self,
        expr: &ExprLoop,
        _: &Facts<'_, ExprLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            [expr.loop_token.span],
            Message::new(Problem::new("loop"), Fix::new("drive an iterator with combinators, const fns excepted")),
        )
    }

    fn on_expr_for_loop(
        &self,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            [expr.for_token.span],
            Message::new(Problem::new("for loop"), Fix::new("drive the iterator with combinators, const fns excepted")),
        )
    }

    fn on_expr_while(
        &self,
        expr: &ExprWhile,
        _: &Facts<'_, ExprWhile>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            [expr.while_token.span],
            Message::new(
                Problem::new("while loop"),
                Fix::new("drive an iterator with combinators, const fns excepted"),
            ),
        )
    }

    fn on_expr_assign(
        &self,
        expr: &ExprAssign,
        _: &Facts<'_, ExprAssign>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            ExpressionFlow::assigned_local(&expr.left).map(|_| expr.eq_token.span),
            Message::new(
                Problem::new("assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        )
    }

    fn on_expr_binary(
        &self,
        expr: &ExprBinary,
        _: &Facts<'_, ExprBinary>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        self.reported(
            ExpressionFlow::assigned_local(&expr.left).and(ExpressionFlow::compound(expr.op)).map(|_| expr.op.span()),
            Message::new(
                Problem::new("compound assignment to a local"),
                Fix::new("build the value in one expression or give its type an updater method"),
            ),
        )
    }

    fn on_local(
        &self,
        local: &Local,
        _: &Facts<'_, Local>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        let declared = MutableLocals::declared_mutable(local)
            .map(|binding| Declared { name: Name::from(&binding.ident), span: local.let_token.span });
        Contributed::parted(Flowed { declared: declared.into_iter().collect(), ..Flowed::default() })
    }

    fn on_expr(
        &self,
        expr: &Expr,
        _: &Facts<'_, Expr>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Flowed<'scan>> {
        let touched = MutableLocals::touched_by(expr).map(Name::from);
        Contributed::parted(Flowed { touched: touched.into_iter().collect(), ..Flowed::default() })
    }
}

impl<'scan> Concludes<'scan> for Flows<'scan> {
    fn violations(self, (): (), flowed: Flowed<'scan>) -> Vec<Violation<'scan>> {
        let mut found = flowed.found;
        found.sort_by_key(|violation| *violation.site());
        found
    }
}

#[derive(Debug)]
struct MutableLocals;

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
        let arguments = writing.and_then(|_| {
            invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ignoring(Ignored::Unparsed)
        });
        arguments?.first().and_then(ExpressionFlow::assigned_local).map(Name::from)
    }
}
