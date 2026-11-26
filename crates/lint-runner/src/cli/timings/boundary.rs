use std::collections::BTreeMap;
use std::fmt;

use lint_core::{Joins, Visits};
use lint_sweep::Visited;

#[derive(Clone, Debug, Default)]
pub(crate) struct Visiting(BTreeMap<String, Visits>);

impl From<Visited> for Visiting {
    fn from(visited: Visited) -> Visiting {
        Visiting(BTreeMap::from([(visited.contributor().to_string(), visited.nodes())]))
    }
}

impl Visiting {
    pub(crate) fn combine(mut self, other: Visiting) -> Visiting {
        other.0.into_iter().for_each(|(contributor, nodes)| {
            let held = self.0.remove(&contributor).map_or(nodes, |earlier| earlier.combine(nodes));
            self.0.insert(contributor, held);
        });
        self
    }
}

impl fmt::Display for Visiting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "nodes visited per contributor:")?;
        self.0.iter().try_for_each(|(contributor, nodes)| writeln!(formatter, "{nodes:>7}  {contributor}"))
    }
}
