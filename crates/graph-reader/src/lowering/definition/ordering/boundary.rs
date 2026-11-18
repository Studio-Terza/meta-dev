use std::collections::BTreeMap;

use graph::CallOrder;
use lint_sweep::Address;

use super::super::Made;

#[derive(Debug)]
pub(crate) struct CallOrders<'made> {
    positions: BTreeMap<Address, usize>,
    invoked: Vec<&'made Made>,
    through: Vec<usize>,
}

impl<'made> From<&'made [Made]> for CallOrders<'made> {
    fn from(made: &'made [Made]) -> CallOrders<'made> {
        let positions = made.iter().enumerate().map(|(index, found)| (found.at, index)).collect();
        let invoked: Vec<&Made> = made.iter().filter(|found| found.invocation.is_some()).collect();
        let through = made
            .iter()
            .scan(0, |seen, found| {
                *seen += usize::from(found.invocation.is_some());
                Some(*seen)
            })
            .collect();
        CallOrders { positions, invoked, through }
    }
}

impl CallOrders<'_> {
    pub(crate) fn order(&self, found: &Made) -> Option<CallOrder> {
        let completed =
            found.receiving().map(|at| self.positions.get(&at).and_then(|index| self.through.get(*index)))?;
        let counted = completed.copied().unwrap_or(0);
        CallOrder::latest(self.invoked.get(..counted)?)
    }
}
