use std::any;
use std::collections::BTreeMap;
use std::fmt;

use algebra::{Monoid, Semigroup};
use lint_core::Visits;
use lint_sweep::Visited;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Named(fn() -> &'static str);

impl Named {
    pub(crate) const fn spelled<Lint: ?Sized>() -> Named {
        Named(any::type_name::<Lint>)
    }
}

impl fmt::Display for Named {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str((self.0)())
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Visiting(BTreeMap<String, Visits>);

impl From<Visited> for Visiting {
    fn from(visited: Visited) -> Visiting {
        Visiting(BTreeMap::from([(visited.contributor().to_string(), visited.nodes())]))
    }
}

impl Semigroup for Visiting {
    fn combine(self, other: Visiting) -> Visiting {
        Visiting(self.0.combine(other.0))
    }
}

impl Monoid for Visiting {}

impl fmt::Display for Visiting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "nodes visited per contributor:")?;
        self.0.iter().try_for_each(|(contributor, nodes)| writeln!(formatter, "{nodes:>7}  {contributor}"))
    }
}
