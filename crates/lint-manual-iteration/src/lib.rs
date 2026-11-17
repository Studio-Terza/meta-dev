use lint_core::Headline;
use lint_core::{Drawn, Fold, Folding, Visit};
use lint_file::{Reads, SweptLint};
use lint_sweep::{Attaching, Attachment, Descend, Facts, Finds, Kind, Scoped};
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

    fn watcher(file: &SourceFile) -> Box<dyn Finds<'_> + '_> {
        Box::new(Scoped::new(Folding::from(Iterations { path: AsRef::<Path>::as_ref(file.head()) })))
    }
}

#[derive(Debug)]
struct Iterations<'scan> {
    path: &'scan Path,
}

#[derive(Debug, Default)]
struct Iterated<'scan> {
    violations: Vec<Violation<'scan>>,
    opening: Vec<Attachment<IteratorCalls>>,
    held: Vec<Vec<Violation<'scan>>>,
}

impl<'scan> Iterations<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }
}

impl<'scan> Iterated<'scan> {
    fn opened(self, opened: Option<Attachment<IteratorCalls>>) -> Iterated<'scan> {
        match opened {
            Some(attachment) => Iterated {
                violations: Vec::new(),
                opening: self.opening.into_iter().chain([attachment]).collect(),
                held: self.held.into_iter().chain([self.violations]).collect(),
            },
            None => self,
        }
    }
}

impl<'scan> From<Iterated<'scan>> for Vec<Violation<'scan>> {
    fn from(iterated: Iterated<'scan>) -> Vec<Violation<'scan>> {
        iterated.violations
    }
}

impl<'scan> Fold for Iterations<'scan> {
    type Found = Iterated<'scan>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::Macro,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::Local,
            Kind::ExprForLoop,
        ]
    }

    fn on_item_mod(&self, found: Iterated<'scan>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_impl(
        &self,
        found: Iterated<'scan>,
        _: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
    ) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_item_trait(
        &self,
        found: Iterated<'scan>,
        _: &ItemTrait,
        facts: &Facts<'_, ItemTrait>,
    ) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_macro(&self, found: Iterated<'scan>, _: &Macro, _: &Facts<'_, Macro>) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::Expand)
    }

    fn on_item_fn(&self, found: Iterated<'scan>, _: &ItemFn, facts: &Facts<'_, ItemFn>) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_impl_item_fn(
        &self,
        found: Iterated<'scan>,
        _: &ImplItemFn,
        facts: &Facts<'_, ImplItemFn>,
    ) -> Visit<Iterated<'scan>> {
        Visit::new(found, Descend::from(facts.gate()))
    }

    fn on_local(&self, found: Iterated<'scan>, local: &Local, _: &Facts<'_, Local>) -> Visit<Iterated<'scan>> {
        let init = Some(local)
            .filter(|candidate| matches!(&candidate.pat, Pat::Ident(binding) if binding.mutability.is_some()))
            .and_then(|mutable| mutable.init.as_ref());
        let opened = init.map(|initial| Attachment::reaching(&*initial.expr, IteratorCalls { local: local.span() }));
        Visit::descending(found.opened(opened))
    }

    fn on_expr_for_loop(
        &self,
        found: Iterated<'scan>,
        expr: &ExprForLoop,
        _: &Facts<'_, ExprForLoop>,
    ) -> Visit<Iterated<'scan>> {
        let finding = ManualIteration::accumulator(&expr.body).map(|_| {
            self.violation(
                expr.for_token.span,
                Message::new(
                    Problem::new("loop that only accumulates"),
                    Fix::new("extend or collect from an iterator instead"),
                ),
            )
        });
        Visit::descending(Iterated { violations: found.violations.into_iter().chain(finding).collect(), ..found })
    }
}

impl<'scan> Attaching<IteratorCalls> for Iterations<'scan> {
    const ATTACHED: &'static [Kind] = &[Kind::Expr, Kind::ExprMethodCall];

    fn attaching(&self, found: Iterated<'scan>) -> Drawn<Iterated<'scan>, Vec<Attachment<IteratorCalls>>> {
        Drawn::new(Iterated { opening: Vec::new(), ..found }, found.opening)
    }

    fn detached(&self, found: Iterated<'scan>, calls: Constructed) -> Iterated<'scan> {
        let mut held = found.held;
        let outer = held.pop().unwrap_or_default();
        let finding = calls.found.and(calls.local).map(|span| {
            self.violation(
                span,
                Message::new(
                    Problem::new("mutable iterator local"),
                    Fix::new("drive it with combinators such as format, collect, or fold"),
                ),
            )
        });
        Iterated { violations: outer.into_iter().chain(finding).chain(found.violations).collect(), held, ..found }
    }
}

#[derive(Debug)]
struct IteratorCalls {
    local: Span,
}

#[derive(Debug, Default)]
struct Constructed {
    found: Option<()>,
    local: Option<Span>,
}

impl Fold for IteratorCalls {
    type Found = Constructed;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ExprMethodCall]
    }

    fn finished(&self, found: Constructed) -> Constructed {
        Constructed { local: Some(self.local), ..found }
    }

    fn on_expr_method_call(
        &self,
        found: Constructed,
        _: &ExprMethodCall,
        facts: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Constructed> {
        let constructed = facts.name().and_then(|name| name.matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS));
        Visit::descending(Constructed { found: found.found.or(constructed.map(|_| ())), ..found })
    }
}
