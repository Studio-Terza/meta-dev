use std::collections::BTreeSet;

use crate::crate_summary::cut::condensed::TangleIndex;
use crate::crate_summary::cut::conductance::Conductance;

#[derive(Clone, Debug)]
pub struct Split {
    conductance: Conductance,
    inside: BTreeSet<TangleIndex>,
    outside: BTreeSet<TangleIndex>,
}

impl Split {
    pub fn new(conductance: Conductance, inside: BTreeSet<TangleIndex>, outside: BTreeSet<TangleIndex>) -> Split {
        Split { conductance, inside, outside }
    }

    pub fn conductance(&self) -> Conductance {
        self.conductance
    }

    pub fn inside(&self) -> &BTreeSet<TangleIndex> {
        &self.inside
    }

    pub fn into_outside(self) -> BTreeSet<TangleIndex> {
        self.outside
    }
}
