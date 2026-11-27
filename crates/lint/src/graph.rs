use std::sync::Arc;

use syn::visit::Visit;

use crate::finding::{Finding, Rule};
use crate::source::Source;
use crate::workspace::Package;

pub(crate) trait Graph<'file>: Sized + for<'ast> Visit<'ast> {
    const RULE: Rule;

    type Fact: Send;

    fn new(source: &'file Source) -> Self;

    fn facts(self) -> Vec<Self::Fact>;

    fn reduce(facts: Vec<Self::Fact>, packages: &[Arc<Package>]) -> Vec<Finding>;
}
