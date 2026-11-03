mod consuming;
mod hyperedge;
mod label;

use std::collections::BTreeMap;

use lint_core::Site;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::edge::Edge;
use crate::end::Node;
use crate::workspace_types::WorkspaceTypes;
pub(crate) use consuming::{Consuming, Moves};
pub(crate) use hyperedge::Hyperedge;
pub(crate) use label::{Interface, Label};

#[derive(Debug)]
pub(crate) struct Graph<'found> {
    nodes: BTreeMap<Node, &'found Site>,
    edges: Vec<Edge<'found>>,
    hyperedges: Vec<Hyperedge<'found>>,
}

impl<'found> Graph<'found> {
    pub(crate) fn built(facts: &'found Facts, transitions: &'found Transitions) -> Graph<'found> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'found>> = transitions.all().iter().map(|found| types.edge(found)).collect();
        let hyperedges = edges.iter().filter_map(Hyperedge::of_edge).collect();
        let nodes = facts.types().iter().fold(BTreeMap::new(), |mut nodes, found| {
            nodes.entry(Node::declared(found.as_ref(), found.as_ref())).or_insert(AsRef::<Site>::as_ref(found));
            nodes
        });
        Graph { nodes, edges, hyperedges }
    }

    pub(crate) fn nodes(&self) -> &BTreeMap<Node, &'found Site> {
        &self.nodes
    }

    pub(crate) fn edges(&self) -> &[Edge<'found>] {
        &self.edges
    }

    pub(crate) fn hyperedges(&self) -> &[Hyperedge<'found>] {
        &self.hyperedges
    }
}
