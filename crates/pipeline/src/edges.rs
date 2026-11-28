use std::collections::{BTreeMap, BTreeSet};

use crate::Measure;

#[derive(Clone, Debug, PartialEq, Eq, Measure)]
pub struct Edges<From: Ord, To: Ord>(BTreeMap<From, BTreeSet<To>>);

impl<From: Ord, To: Ord> Edges<From, To> {
    #[must_use]
    pub fn node(from: From) -> Edges<From, To> {
        Edges(BTreeMap::from([(from, BTreeSet::new())]))
    }

    #[must_use]
    pub fn edge(from: From, to: To) -> Edges<From, To> {
        Edges(BTreeMap::from([(from, BTreeSet::from([to]))]))
    }

    pub fn targets(&self, from: &From) -> Option<&BTreeSet<To>> {
        self.0.get(from)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &From> {
        self.0.keys()
    }

    #[must_use]
    pub fn reached(&self) -> BTreeSet<&To> {
        self.0.values().flatten().collect()
    }

    pub fn bare(&self) -> impl Iterator<Item = &From> {
        self.0.iter().filter(|(_, targets)| targets.is_empty()).map(|(from, _)| from)
    }

    #[must_use]
    pub fn crowded(self) -> Edges<From, To> {
        Edges(self.0.into_iter().filter(|(_, targets)| targets.len() > 1).collect())
    }
}

impl<From: Ord + Clone, To: Ord + Clone> Edges<From, To> {
    #[must_use]
    pub fn neighbourhoods(&self) -> Edges<BTreeSet<To>, From> {
        Edges::total(self.0.iter().map(|(from, targets)| Edges::edge(targets.clone(), from.clone())))
    }
}
