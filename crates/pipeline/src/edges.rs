use std::collections::{BTreeMap, BTreeSet};

use crate::Measure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edges<From: Ord, To: Ord>(BTreeMap<From, BTreeSet<To>>);

impl<From: Ord, To: Ord> Default for Edges<From, To> {
    fn default() -> Edges<From, To> {
        Edges(BTreeMap::new())
    }
}

impl<From: Ord, To: Ord> Measure for Edges<From, To> {
    fn joined(self, other: Edges<From, To>) -> Edges<From, To> {
        Edges(self.0.joined(other.0))
    }
}

impl<From: Ord, To: Ord> Edges<From, To> {
    #[must_use]
    pub fn edge(from: From, to: To) -> Edges<From, To> {
        Edges(BTreeMap::from([(from, BTreeSet::from([to]))]))
    }

    pub fn targets(&self, from: &From) -> Option<&BTreeSet<To>> {
        self.0.get(from)
    }
}
