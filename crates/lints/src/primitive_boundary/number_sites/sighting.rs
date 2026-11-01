use lint_core::Violation;

use super::rank::Rank;

pub struct Sighting {
    rank: Rank,
    violation: Violation,
}

impl Sighting {
    pub fn new(rank: Rank, violation: Violation) -> Sighting {
        Sighting { rank, violation }
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub fn into_violation(self) -> Violation {
        self.violation
    }
}
