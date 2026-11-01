mod density;

use std::fmt;

use super::component::Component;
use super::count::Count;
use density::Density;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Measures {
    nodes: Count,
    edges: Count,
    density: Density,
    largest: Component,
}

impl Measures {
    pub(crate) fn counted(nodes: Count, edges: Count, largest: Component) -> Measures {
        Measures { nodes, edges, density: Density::between(edges, nodes), largest }
    }
}

impl fmt::Display for Measures {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} nodes, {} edges, density {}, largest strongly connected component {}",
            self.nodes, self.edges, self.density, self.largest
        )
    }
}
