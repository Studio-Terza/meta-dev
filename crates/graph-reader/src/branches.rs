use syntax::{BinOp, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprTry, ExprWhile, Item};

use body_lifts::Branching;
use lint_core::{Fold, Visit};
use lint_sweep::{Descend, Facts, Kind};
use newtype::Counter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Propagation {
    Branch,
    Ambient,
}

#[derive(Debug)]
pub struct Branches {
    propagation: Propagation,
}

#[derive(Debug, Default)]
pub struct Branched {
    extra: Branching,
}

impl From<Branched> for Branching {
    fn from(branched: Branched) -> Branching {
        Branching::new(1).plus(branched.extra)
    }
}

impl Branches {
    #[must_use]
    pub fn new(propagation: Propagation) -> Branches {
        Branches { propagation }
    }
}

impl Branched {
    fn counted(self, extra: Branching) -> Visit<Branched> {
        Visit::descending(Branched { extra: self.extra.plus(extra) })
    }
}

impl Fold for Branches {
    type Found = Branched;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Item,
            Kind::ExprIf,
            Kind::ExprWhile,
            Kind::ExprForLoop,
            Kind::ExprTry,
            Kind::ExprMatch,
            Kind::ExprBinary,
        ]
    }

    fn on_item(&self, found: Branched, _: &Item, _: &Facts<'_, Item>) -> Visit<Branched> {
        Visit::new(found, Descend::Over)
    }

    fn on_expr_if(&self, found: Branched, _: &ExprIf, _: &Facts<'_, ExprIf>) -> Visit<Branched> {
        found.counted(Branching::new(1))
    }

    fn on_expr_while(&self, found: Branched, _: &ExprWhile, _: &Facts<'_, ExprWhile>) -> Visit<Branched> {
        found.counted(Branching::new(1))
    }

    fn on_expr_for_loop(&self, found: Branched, _: &ExprForLoop, _: &Facts<'_, ExprForLoop>) -> Visit<Branched> {
        found.counted(Branching::new(1))
    }

    fn on_expr_try(&self, found: Branched, _: &ExprTry, _: &Facts<'_, ExprTry>) -> Visit<Branched> {
        found.counted(Branching::new(usize::from(self.propagation == Propagation::Branch)))
    }

    fn on_expr_match(&self, found: Branched, matched: &ExprMatch, _: &Facts<'_, ExprMatch>) -> Visit<Branched> {
        found.counted(Branching::new(matched.arms.len().saturating_sub(1)))
    }

    fn on_expr_binary(&self, found: Branched, binary: &ExprBinary, _: &Facts<'_, ExprBinary>) -> Visit<Branched> {
        found.counted(Branching::new(usize::from(matches!(binary.op, BinOp::And(_) | BinOp::Or(_)))))
    }
}
