use std::ptr;

use syn::spanned::Spanned;
use syn::{
    Block, Expr, ExprForLoop, ExprMethodCall, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Macro, Pat, Stmt,
};

use crate::finding::{Found, Rule};
use crate::gate::Gate;
use crate::hooks::{Descent, Expanded, Hooks};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::terms::{Count, Fix};

#[derive(Debug, Default)]
pub(crate) struct ManualIteration {
    found: Vec<Found>,
    counting: Vec<Counting>,
}

#[derive(Debug)]
struct Counting {
    local: *const Local,
    init: *const Expr,
    calls: Count,
    reading: Reading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reading {
    Waiting,
    Inside,
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
    const ACCUMULATING_FIX: Fix = Fix("extend or collect from an iterator instead");
    const COUNTING_FIX: Fix = Fix("drive it with combinators such as format, collect, or fold");

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

    fn entered(&mut self, expr: &Expr) {
        self.counting.last_mut().filter(|counting| ptr::eq(counting.init, expr)).map_or_default(Counting::reached);
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
    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        Descent::from(Gate::of(&module.attrs).entered())
    }

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
            Found::spanned(expr.for_token.span, "loop that only accumulates", ManualIteration::ACCUMULATING_FIX)
        }));
        Descent::Into
    }

    fn local(&mut self, local: &Local) -> Descent {
        let counting = ManualIteration::mutable_init(local).map(|init| Counting {
            local,
            init,
            calls: Count::default(),
            reading: Reading::Waiting,
        });
        self.counting.extend(counting);
        Descent::Into
    }

    fn left_local(&mut self, local: &Local) {
        let counted = self
            .counting
            .pop_if(|counting| ptr::eq(counting.local, local))
            .filter(|ended| ended.calls > Count::default());
        counted.map_or_default(|_ended| {
            let found = Found::spanned(local.span(), "mutable iterator local", ManualIteration::COUNTING_FIX);
            self.found.push(found);
        });
    }

    fn expr(&mut self, expr: &Expr) -> Descent {
        self.entered(expr);
        Descent::Into
    }

    fn expr_method_call(&mut self, call: &ExprMethodCall) -> Descent {
        let constructs = ManualIteration::ITERATOR_CONSTRUCTORS.iter().find(|name| call.method == name).map(|_| ());
        self.counting.iter_mut().for_each(|counting| counting.counted(constructs));
        Descent::Into
    }
}

impl Counting {
    fn reached(&mut self) {
        self.reading = Reading::Inside;
    }

    fn counted(&mut self, constructs: Option<()>) {
        let inside = constructs.filter(|()| self.reading == Reading::Inside);
        self.calls = inside.map_or(self.calls, |()| self.calls.incremented());
    }
}

impl<'file> Style<'file> for ManualIteration {
    const RULE: Rule = Rule::ManualIteration;
    const READS: Reads = Reads::Shipped;

    fn new(_: &'file Source) -> ManualIteration {
        ManualIteration::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
