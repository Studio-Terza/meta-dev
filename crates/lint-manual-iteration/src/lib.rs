use lint_core::Headline;
use lint_file::{Reads, SweptLint};
use lint_sweep::{Attaches, Attachment, Descend, Facts, Finds, Gathers, Kind, Scoped, Watch, Watching};
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
        Box::new(Scoped::new(Iterations {
            path: AsRef::<Path>::as_ref(file.head()),
            violations: Vec::new(),
            opening: Vec::new(),
            held: Vec::new(),
        }))
    }
}

#[derive(Debug)]
struct Iterations<'scan> {
    path: &'scan Path,
    violations: Vec<Violation<'scan>>,
    opening: Vec<Attachment<IteratorCalls>>,
    held: Vec<Vec<Violation<'scan>>>,
}

impl<'scan> Iterations<'scan> {
    fn violation(&self, span: Span, message: Message) -> Violation<'scan> {
        Violation::new(Site::new(self.path, LineNumber::from(span)), message)
    }
}

impl<'ast> Watch<'ast> for Iterations<'_> {
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

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_impl(&mut self, _: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_item_trait(&mut self, _: &'ast ItemTrait, facts: &Facts<'_, ItemTrait>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_macro(&mut self, _: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        Descend::Expand
    }

    fn on_item_fn(&mut self, _: &'ast ItemFn, facts: &Facts<'_, ItemFn>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_impl_item_fn(&mut self, _: &'ast ImplItemFn, facts: &Facts<'_, ImplItemFn>) -> Descend {
        Descend::from(facts.gate())
    }

    fn on_local(&mut self, local: &'ast Local, _: &Facts<'_, Local>) -> Descend {
        let init = Some(local)
            .filter(|candidate| matches!(&candidate.pat, Pat::Ident(binding) if binding.mutability.is_some()))
            .and_then(|mutable| mutable.init.as_ref());
        let opened =
            init.map(|found| Attachment::reaching(&*found.expr, IteratorCalls { found: None, local: local.span() }));
        self.held.extend(opened.iter().map(|_| std::mem::take(&mut self.violations)));
        self.opening.extend(opened);
        Descend::Into
    }

    fn on_expr_for_loop(&mut self, expr: &'ast ExprForLoop, _: &Facts<'_, ExprForLoop>) -> Descend {
        let finding = ManualIteration::accumulator(&expr.body).map(|_| {
            self.violation(
                expr.for_token.span,
                Message::new(
                    Problem::new("loop that only accumulates"),
                    Fix::new("extend or collect from an iterator instead"),
                ),
            )
        });
        self.violations.extend(finding);
        Descend::Into
    }
}

impl Attaches<IteratorCalls> for Iterations<'_> {
    const ATTACHED: &'static [Kind] = &[Kind::Expr, Kind::ExprMethodCall];

    fn attaching(&mut self) -> Vec<Attachment<IteratorCalls>> {
        std::mem::take(&mut self.opening)
    }

    fn detached(&mut self, calls: IteratorCalls) {
        let inside = std::mem::take(&mut self.violations);
        self.violations = self.held.pop().unwrap_or_default();
        let finding = calls.found.map(|()| {
            self.violation(
                calls.local,
                Message::new(
                    Problem::new("mutable iterator local"),
                    Fix::new("drive it with combinators such as format, collect, or fold"),
                ),
            )
        });
        self.violations.extend(finding);
        self.violations.extend(inside);
    }
}

impl<'file> Finds<'file> for Iterations<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        self.violations
    }
}

#[derive(Debug)]
struct IteratorCalls {
    found: Option<()>,
    local: Span,
}

impl Gathers<IteratorCalls> for IteratorCalls {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        std::iter::once::<&mut Watching<'_>>(self)
    }

    fn gathered(self) -> IteratorCalls {
        self
    }
}

impl<'ast> Watch<'ast> for IteratorCalls {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ExprMethodCall]
    }

    fn on_expr_method_call(&mut self, _: &'ast ExprMethodCall, facts: &Facts<'_, ExprMethodCall>) -> Descend {
        let constructed = facts.name().and_then(|name| name.matches::<Whole>(&ManualIteration::ITERATOR_CONSTRUCTORS));
        self.found = self.found.or(constructed.map(|_| ()));
        Descend::Into
    }
}
