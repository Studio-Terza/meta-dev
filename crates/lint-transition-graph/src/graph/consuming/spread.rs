use std::collections::BTreeSet;

use super::Consuming;
use crate::end::Node;

#[derive(Debug)]
pub(crate) struct Spread<'graph> {
    reached: BTreeSet<&'graph Node<'graph>>,
    fresh: BTreeSet<&'graph Node<'graph>>,
}

impl<'graph> Spread<'graph> {
    pub(crate) fn new(
        reached: BTreeSet<&'graph Node<'graph>>,
        fresh: BTreeSet<&'graph Node<'graph>>,
    ) -> Spread<'graph> {
        Spread { reached, fresh }
    }

    pub(crate) fn grown(self, consuming: &Consuming<'graph, '_>) -> BTreeSet<&'graph Node<'graph>> {
        match self.fresh.first() {
            Some(_) => consuming.grown(self.reached, &self.fresh),
            None => self.reached,
        }
    }
}
