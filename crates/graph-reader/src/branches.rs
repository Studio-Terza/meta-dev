use syn::{BinOp, Block, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprTry, ExprWhile, Item};

use body_lifts::Branching;
use lint_sweep::{Descend, Kind, Sweep, Watch};
use newtype::Counter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Propagation {
    Branch,
    Ambient,
}

#[derive(Debug)]
pub struct Branches {
    score: Branching,
    propagation: Propagation,
}

impl Branches {
    #[must_use]
    pub fn new(propagation: Propagation) -> Branches {
        Branches { score: Branching::new(1), propagation }
    }

    #[must_use]
    pub fn scored(body: &Block, propagation: Propagation) -> Branching {
        let mut scan = Branches::new(propagation);
        Sweep::over_block(body, vec![&mut scan]);
        scan.score
    }

    #[must_use]
    pub fn score(self) -> Branching {
        self.score
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

    fn on_item(&mut self, _: &'ast Item) -> Descend {
        Descend::Over
    }

    fn on_expr_if(&mut self, _: &'ast ExprIf) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_while(&mut self, _: &'ast ExprWhile) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_for_loop(&mut self, _: &'ast ExprForLoop) -> Descend {
        self.counted(Branching::new(1))
    }

    fn on_expr_try(&mut self, _: &'ast ExprTry) -> Descend {
        self.counted(Branching::new(usize::from(self.propagation == Propagation::Branch)))
    }

    fn on_expr_match(&mut self, found: &'ast ExprMatch) -> Descend {
        self.counted(Branching::new(found.arms.len().saturating_sub(1)))
    }

    fn on_expr_binary(&mut self, found: &'ast ExprBinary) -> Descend {
        self.counted(Branching::new(usize::from(matches!(found.op, BinOp::And(_) | BinOp::Or(_)))))
    }
}
