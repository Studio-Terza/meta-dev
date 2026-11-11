use design_model::{Facet, Title};

use crate::{Edge, Label, NodeRef, Visit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proposal {
    Anonymous(NodeRef),
    Labelled(Edge<Label, NodeRef>),
}

impl Proposal {
    pub(crate) fn visited<V: Visit>(&self, visitor: V, facet: Facet, title: &Title) -> V::Output {
        match self {
            Proposal::Anonymous(refines) => visitor.propose(None, refines, facet, title),
            Proposal::Labelled(edge) => edge.ends(|label, refines| visitor.propose(Some(label), refines, facet, title)),
        }
    }
}

impl From<Label> for NodeRef {
    fn from(label: Label) -> NodeRef {
        NodeRef::Labelled(label)
    }
}
