use std::collections::BTreeSet;

use newtype::Counter;

use super::leg::Leg;
use super::stops::Stop;
use crate::count::Count;
use crate::edge::Edge;
use equivalence::Keyed;

#[derive(Debug)]
pub(super) struct Walk<'graph, 'found, 'file> {
    edge: &'graph Edge<'found, 'file>,
    called: Vec<&'graph Edge<'found, 'file>>,
    stops: Vec<Stop>,
}

impl<'graph, 'found, 'file> Walk<'graph, 'found, 'file> {
    const LEAST_HOPS: usize = 3;
    const LEAST_DISTINCT: usize = 2;

    pub(super) fn new(
        edge: &'graph Edge<'found, 'file>,
        called: Vec<&'graph Edge<'found, 'file>>,
        stops: Vec<Stop>,
    ) -> Walk<'graph, 'found, 'file> {
        Walk { edge, called, stops }
    }

    pub(super) fn legs(&self) -> impl Iterator<Item = Leg<'_, 'graph, 'found, 'file>> {
        let length = self.stops.len();
        (0..length)
            .flat_map(move |from| {
                (from + Self::LEAST_HOPS..=length).map(move |to| Leg::new(self, Count::new(from), Count::new(to)))
            })
            .filter(|leg| Self::varied(leg.key()).is_some())
    }

    pub(super) fn edge(&self) -> &'graph Edge<'found, 'file> {
        self.edge
    }

    pub(super) fn called(&self, from: Count, to: Count) -> &[&'graph Edge<'found, 'file>] {
        self.called.get(from.count()..to.count()).unwrap_or_default()
    }

    pub(super) fn stops(&self, from: Count, to: Count) -> &[Stop] {
        self.stops.get(from.count()..to.count()).unwrap_or_default()
    }

    pub(super) fn resolved(&self) -> Count {
        Count::new(self.called.len())
    }

    fn varied(stops: &[Stop]) -> Option<&[Stop]> {
        Some(stops).filter(|candidate| candidate.iter().collect::<BTreeSet<&Stop>>().len() >= Self::LEAST_DISTINCT)
    }
}
