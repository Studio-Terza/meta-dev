use std::fmt;

use lint_layout::Fix;
use newtype::Counter;

use crate::count::Count;
use crate::link::Link;
use crate::tangle::Tangle;

#[derive(Debug)]
pub(crate) struct Tally {
    definitions: Count,
    edges: Count,
    tangles: Count,
    tangled: Count,
}

impl Tally {
    pub(crate) fn counted(definitions: Count, links: &[Link<'_, '_>], found: &[Tangle<'_, '_>]) -> Tally {
        Tally {
            definitions,
            edges: Count::new(links.len()),
            tangles: Count::new(found.len()),
            tangled: Count::total(found.iter().map(Tangle::size)),
        }
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {} definitions, {} edges, {} tangles holding {} definitions",
            self.definitions, self.edges, self.tangles, self.tangled
        )
    }
}

impl lint_layout::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("make the definition graph acyclic, with self-recursion only through a pointer")
    }
}
