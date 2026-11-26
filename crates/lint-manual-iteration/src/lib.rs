use algebra::{Monoid, Semigroup};
use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Concludes, Contribute, Contributed, Contributing, Contribution, Descend, Facts, Kind, Scope};
use std::path::Path;
use syntax::Span;
use syntax::Spanned;

use syntax::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::{Name, Whole, Word, words};

#[derive(Debug)]
pub struct ManualIteration;

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [Word<'static>; 16] = words![
        "iter",
        "iter_mut",
        "into_iter",
        "chars",
        "bytes",
        "lines",
        "split",
        "split_whitespace",
        "rev",
        "skip",
        "take",
        "zip",
        "enumerate",
        "map",
        "filter",
        "peekable",
    ];
    const ACCUMULATORS: [Word<'static>; 4] = words!["push", "extend", "insert", "push_str"];

    fn accumulator(block: &Block) -> Option<&ExprMethodCall> {
        Some(block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Self::accumulating)
    }

    fn accumulating(statement: &Stmt) -> Option<&ExprMethodCall> {
        match statement {
            Stmt::Expr(Expr::If(branch), _) => Some(branch)
                .filter(|candidate| candidate.else_branch.is_none())
                .and_then(|bare_if| Self::accumulator(&bare_if.then_branch)),
            Stmt::Expr(Expr::MethodCall(call), _) => Self::accumulated(call),
            _ => None,
        }
    }

    fn accumulated(call: &ExprMethodCall) -> Option<&ExprMethodCall> {
        Some(call)
            .filter(|candidate| matches!(*candidate.receiver, Expr::Path(_) | Expr::Field(_)))
            .and_then(|on_place| Name::from(&on_place.method).matches::<Whole>(&Self::ACCUMULATORS).map(|_| on_place))
    }
}

impl SweptLint for ManualIteration {
    const HEADLINE: Headline = Headline::new("iterators are driven by combinators, not by hand");
    const READS: Reads = Reads::Shipped;

    fn watcher(file: &SourceFile) -> Contributing<impl Concludes<'_>> {
        Contributing::new(Iterations { path: AsRef::<Path>::as_ref(file.head()) })
    }
}

#[derive(Debug)]
struct Iterations<'scan> {
    path: &'scan Path,
}

#[derive(Debug, Default)]
struct Iterated<'scan> {
    found: Vec<Violation<'scan>>,
    calls: Vec<Span>,
}

impl Semigroup for Iterated<'_> {
    fn combine(self, other: Self) -> Self {
        Iterated { found: self.found.combine(other.found), calls: self.calls.combine(other.calls) }
    }
}

impl Monoid for Iterated<'_> {}

impl Iterated<'_> {
    fn constructs(&self, init: &Expr) -> Option<()> {
        let range = init.span().byte_range();
        self.calls.iter().find(|call| range.contains(&call.byte_range().start)).map(|_| ())
    }
}

impl<'scan> Iterations<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }

    fn mutable_init(local: &Local) -> Option<&Expr> {
        Some(local)
            .filter(|candidate| matches!(&candidate.pat, Pat::Ident(binding) if binding.mutability.is_some()))
            .and_then(|mutable| mutable.init.as_ref())
            .map(|init| &*init.expr)
    }
}

impl<'scan> Contribute for Iterations<'scan> {
    type Open = ();
    type Part = Iterated<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::ExprForLoop,
            Kind::ExprMethodCall,
        ]
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::Local)] }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_item_trait(
        &self,
        _: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_macro(
        &self,
        _: &Macro,
        _: &Facts<'_, Macro>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::Expand)
    }

    fn on_item_fn(
        &self,
        _: &ItemFn,
        facts: &Facts<'_, ItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn on_impl_item_fn(
        &self,
        _: &ImplItemFn,
        facts: &Facts<'_, ImplItemFn>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn off_local(
        &self,
        local: &Local,
        _: &Facts<'_, Local>,
        inner: Contribution<(), Iterated<'scan>>,
    ) -> Iterated<'scan> {
        inner.concluded(|(), iterated| {
            let constructed =
                Self::mutable_init(local).and_then(|init| iterated.constructs(init)).map(|()| local.span());
            let finding = constructed.map(|span| {
                self.violation(
                    span,
                    Message::new(
                        Problem::new("mutable iterator local"),
                        Fix::new("drive it with combinators such as format, collect, or fold"),
                    ),
                )
            });
            Iterated { found: iterated.found.into_iter().chain(finding).collect(), calls: iterated.calls }
        })
    }

    fn on_expr_for_loop(
        &self,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        let finding = ManualIteration::accumulator(&expr.body).map(|_| {
            self.violation(
                expr.for_token.span,
                Message::new(
                    Problem::new("loop that only accumulates"),
                    Fix::new("extend or collect from an iterator instead"),
                ),
            )
        });
        Contributed::parted(Iterated { found: finding.into_iter().collect(), calls: Vec::new() })
    }

    fn on_expr_method_call(
        &self,
        call: &ExprMethodCall,
        facts: &Facts<'_, ExprMethodCall>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), Iterated<'scan>> {
        let constructed = facts.name().and_then(|name| name.matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS));
        let span = constructed.map(|_| call.method.span());
        Contributed::parted(Iterated { found: Vec::new(), calls: span.into_iter().collect() })
    }
}

impl<'scan> Concludes<'scan> for Iterations<'scan> {
    fn violations(self, (): (), iterated: Iterated<'scan>) -> Vec<Violation<'scan>> {
        let mut found = iterated.found;
        found.sort_by_key(|violation| *violation.site());
        found
    }
}
