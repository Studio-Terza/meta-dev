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
pub(crate) struct Graph<'found, 'file> {
    nodes: BTreeMap<Node, Site<'file>>,
    edges: Vec<Edge<'found, 'file>>,
    hyperedges: Vec<Hyperedge<'found, 'file>>,
}

impl<'found, 'file> Graph<'found, 'file> {
    pub(crate) fn built(facts: &Facts<'file>, transitions: &'found Transitions<'file>) -> Graph<'found, 'file> {
        let types = WorkspaceTypes::declared(facts, transitions);
        let edges: Vec<Edge<'found, 'file>> = transitions.all().iter().map(|found| types.edge(found)).collect();
        let hyperedges = edges.iter().filter_map(Hyperedge::of_edge).collect();
        let nodes = facts.types().iter().fold(BTreeMap::new(), |mut nodes, found| {
            nodes.entry(Node::declared(found.as_ref(), found.as_ref())).or_insert(*AsRef::<Site<'file>>::as_ref(found));
            nodes
        });
        Graph { nodes, edges, hyperedges }
    }

    pub(crate) fn nodes(&self) -> &BTreeMap<Node, Site<'file>> {
        &self.nodes
    }

    pub(crate) fn edges(&self) -> &[Edge<'found, 'file>] {
        &self.edges
    }

    pub(crate) fn hyperedges(&self) -> &[Hyperedge<'found, 'file>] {
        &self.hyperedges
    }
}
