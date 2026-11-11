use design_model::{NodeEdge, Provenance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Link {
    edge: NodeEdge,
    provenance: Provenance,
}

impl Link {
    pub(crate) fn new(edge: NodeEdge, provenance: Provenance) -> Link {
        Link { edge, provenance }
    }

    #[must_use]
    pub fn edge(self) -> NodeEdge {
        self.edge
    }

    #[must_use]
    pub fn provenance(self) -> Provenance {
        self.provenance
    }
}
