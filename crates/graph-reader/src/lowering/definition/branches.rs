use syntax::{BinOp, ExprBinary, ExprMatch};

use algebra::{Monoid, Semigroup};
use body_lifts::{Branching, Propagated};
use newtype::Counter;

#[derive(Debug, Default)]
pub(crate) struct Branches {
    score: Branching,
    propagated: Propagated,
}

impl Semigroup for Branches {
    fn combine(self, other: Branches) -> Branches {
        Branches { score: self.score.plus(other.score), propagated: self.propagated.plus(other.propagated) }
    }
}

impl Monoid for Branches {}

impl Branches {
    pub(crate) fn score(&self) -> Branching {
        self.score
    }

    pub(crate) fn propagated(&self) -> Propagated {
        self.propagated
    }

    pub(crate) fn matched(found: &ExprMatch) -> Branches {
        Branches { score: Branching::new(found.arms.len().saturating_sub(1)), ..Branches::default() }
    }

    pub(crate) fn joined(found: &ExprBinary) -> Branches {
        let joining = usize::from(matches!(found.op, BinOp::And(_) | BinOp::Or(_)));
        Branches { score: Branching::new(joining), ..Branches::default() }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Branch {
    Taken,
    Propagated,
}

impl From<Branch> for Branches {
    fn from(branch: Branch) -> Branches {
        match branch {
            Branch::Taken => Branches { score: Branching::new(1), ..Branches::default() },
            Branch::Propagated => Branches { propagated: Propagated::new(1), ..Branches::default() },
        }
    }
}
