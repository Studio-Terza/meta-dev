use syntax::{BinOp, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprTry, ExprWhile, Item};

use body_lifts::{Branching, Propagated};
use lint_sweep::{Descend, Facts, Kind, Watch};
use newtype::Counter;

#[derive(Debug)]
pub(crate) struct Branches {
    score: Branching,
    propagated: Propagated,
}

impl Branches {
    pub(crate) fn opened() -> Branches {
        Branches { score: Branching::new(1), propagated: Propagated::default() }
    }

    pub(crate) fn score(&self) -> Branching {
        self.score
    }

    pub(crate) fn propagated(&self) -> Propagated {
        self.propagated
    }

    fn counted(&mut self, extra: Branching) -> Descend {
        self.score = self.score.plus(extra);
        Descend::Into
    }
}

impl<'ast> Watch<'ast> for Branches {
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

    fn on_item(&mut self, _: &'ast Item, _: &Facts<'_, Item>) -> Descend {
        Descend::Over
    }

    fn on_expr_if(&mut self, _: &'ast ExprIf, _: &Facts<'_, ExprIf>) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_while(&mut self, _: &'ast ExprWhile, _: &Facts<'_, ExprWhile>) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_for_loop(&mut self, _: &'ast ExprForLoop, _: &Facts<'_, ExprForLoop>) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_try(&mut self, _: &'ast ExprTry, _: &Facts<'_, ExprTry>) -> Descend {
        self.propagated = self.propagated.plus(Propagated::new(1));
        Descend::Into
    }

    fn on_expr_match(&mut self, found: &'ast ExprMatch, _: &Facts<'_, ExprMatch>) -> Descend {
        self.counted(Branching::new(found.arms.len().saturating_sub(1)))
    }

    fn on_expr_binary(&mut self, found: &'ast ExprBinary, _: &Facts<'_, ExprBinary>) -> Descend {
        self.counted(Branching::new(usize::from(matches!(found.op, BinOp::And(_) | BinOp::Or(_)))))
    }
}
