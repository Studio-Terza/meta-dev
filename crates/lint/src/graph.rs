use std::sync::Arc;

use crate::finding::Finding;
use crate::source::Source;
use crate::workspace::Package;

pub(crate) trait Graph<'file>: Sized {
    type Fact: Send;

    fn new(source: &'file Source) -> Self;

    fn facts(self) -> Vec<Self::Fact>;

    fn reduce(facts: Vec<Self::Fact>, packages: &[Arc<Package>]) -> Vec<Finding>;
}
