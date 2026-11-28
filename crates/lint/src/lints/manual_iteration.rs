use std::ptr;

use syn::spanned::Spanned;
use syn::{Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemTrait, Local, Macro, Pat, Stmt};

use crate::finding::{Findings, Found};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;

#[derive(Debug, Default)]
pub(crate) struct ManualIteration {
    found: Vec<Found>,
    counting: Vec<Counting>,
}

#[derive(Debug)]
struct Counting {
    local: *const Local,
    init: *const Expr,
    at: usize,
    calls: usize,
    reading: Reading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reading {
    Waiting,
    Inside,
    Read,
}

impl ManualIteration {
    const ITERATOR_CONSTRUCTORS: [&'static str; 16] = [
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
    const ACCUMULATORS: [&'static str; 4] = ["push", "extend", "insert", "push_str"];

    fn accumulator(block: &Block) -> Option<&ExprMethodCall> {
        match block.stmts.as_slice() {
            [only] => ManualIteration::accumulating(only),
            _ => None,
        }
    }

    fn accumulating(statement: &Stmt) -> Option<&ExprMethodCall> {
        match statement {
            Stmt::Expr(Expr::If(branch), _) => match branch.else_branch {
                None => ManualIteration::accumulator(&branch.then_branch),
                Some(_) => None,
            },
            Stmt::Expr(Expr::MethodCall(call), _) => Some(call)
                .filter(|candidate| matches!(*candidate.receiver, Expr::Path(_) | Expr::Field(_)))
                .filter(|on_place| ManualIteration::ACCUMULATORS.iter().any(|name| on_place.method == name)),
            _ => None,
        }
    }

    fn read(&mut self, expr: &Expr, reading: Reading) {
        let current = self.counting.last_mut().filter(|counting| ptr::eq(counting.init, expr));
        current.map_or_default(|counting| counting.reached(reading));
    }

    fn mutable_init(local: &Local) -> Option<&Expr> {
        let mutable = match &local.pat {
            Pat::Ident(binding) => binding.mutability.map(|_| local),
            _ => None,
        };
        mutable.and_then(|binding| binding.init.as_ref()).map(|init| &*init.expr)
    }
}

impl Hooks for ManualIteration {
    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        Descent::from(Gate::of(&item.attrs).entered())
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        Descent::from(Gate::of(&item.attrs).entered())
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        Descent::from(Gate::of(&function.attrs).entered())
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        Descent::from(Gate::of(&function.attrs).entered())
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }

    fn expr_for_loop(&mut self, expr: &ExprForLoop) -> Descent {
        self.found.extend(ManualIteration::accumulator(&expr.body).map(|_| {
            Found::spanned(
                expr.for_token.span,
                "loop that only accumulates",
                "extend or collect from an iterator instead",
            )
        }));
        Descent::Into
    }

    fn local(&mut self, local: &Local) -> Descent {
        let at = self.found.len();
        let counting = ManualIteration::mutable_init(local).map(|init| Counting {
            local,
            init,
            at,
            calls: 0,
            reading: Reading::Waiting,
        });
        self.counting.extend(counting);
        Descent::Into
    }

    fn left_local(&mut self, local: &Local) {
        let counted = self.counting.pop_if(|counting| ptr::eq(counting.local, local)).filter(|ended| ended.calls > 0);
        counted.map_or_default(|ended| {
            let found = Found::spanned(
                local.span(),
                "mutable iterator local",
                "drive it with combinators such as format, collect, or fold",
            );
            self.found.insert(ended.at, found);
        });
    }

    fn expr(&mut self, expr: &Expr) -> Descent {
        self.read(expr, Reading::Inside);
        Descent::Into
    }

    fn left_expr(&mut self, expr: &Expr) {
        self.read(expr, Reading::Read);
    }

    fn expr_method_call(&mut self, call: &ExprMethodCall) -> Descent {
        let constructs = ManualIteration::ITERATOR_CONSTRUCTORS.iter().any(|name| call.method == name);
        self.counting.iter_mut().for_each(|counting| counting.counted(constructs));
        Descent::Into
    }
}

impl Counting {
    fn reached(&mut self, reading: Reading) {
        self.reading = reading;
    }

    fn counted(&mut self, constructs: bool) {
        self.calls += usize::from(constructs && self.reading == Reading::Inside);
    }
}

impl<'file> Lint<'file> for ManualIteration {
    const HEADLINE: &'static str = "iterators are driven by combinators, not by hand";
    const READS: Reads = Reads::Shipped;

    type Fact = Findings;

    fn new(_: &'file Source) -> ManualIteration {
        ManualIteration::default()
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(ManualIteration::HEADLINE, source.shown(), self.found)
    }
}
