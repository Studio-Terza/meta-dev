use std::collections::{BTreeMap, BTreeSet};

use crate::Measure;

#[derive(Clone, Debug, PartialEq, Eq, Measure)]
pub struct Edges<From: Ord, To: Ord>(BTreeMap<From, BTreeSet<To>>);

impl<From: Ord, To: Ord> Edges<From, To> {
    #[must_use]
    pub fn edge(from: From, to: To) -> Edges<From, To> {
        Edges(BTreeMap::from([(from, BTreeSet::from([to]))]))
    }

    pub fn targets(&self, from: &From) -> Option<&BTreeSet<To>> {
        self.0.get(from)
    }
}
