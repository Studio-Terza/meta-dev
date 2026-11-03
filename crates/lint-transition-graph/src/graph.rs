mod arc;
mod label;

use std::collections::BTreeMap;

use lint_core::Site;
use transition_facts::Transitions;
use type_facts::Facts;

use crate::edge::Edge;
use crate::end::{Node, Step};
use crate::workspace_types::WorkspaceTypes;
pub(crate) use arc::Arc;
pub(crate) use label::Label;

#[derive(Debug)]
pub(crate) struct Graph<'found> {
    nodes: BTreeMap<Node, &'found Site>,
    edges: Vec<Edge<'found>>,
    arcs: Vec<Arc<'found>>,
}

impl<'found> Graph<'found> {
    pub(crate) fn built(facts: &'found Facts, transitions: &'found Transitions) -> Graph<'found> {
        let types = WorkspaceTypes::declared(facts);
        let edges: Vec<Edge<'found>> = transitions.all().iter().map(|found| types.edge(found)).collect();
        let arcs = edges.iter().flat_map(Arc::of_edge).collect();
        let nodes = facts.types().iter().fold(BTreeMap::new(), |mut nodes, found| {
            nodes.entry(Node::declared(found.as_ref(), found.as_ref())).or_insert(AsRef::<Site>::as_ref(found));
            nodes
        });
        Graph { nodes, edges, arcs }
    }

    pub(crate) fn nodes(&self) -> &BTreeMap<Node, &'found Site> {
        &self.nodes
    }

    pub(crate) fn edges(&self) -> &[Edge<'found>] {
        &self.edges
    }

    pub(crate) fn arcs(&self) -> &[Arc<'found>] {
        &self.arcs
    }

    pub(crate) fn leaving(&self) -> BTreeMap<&Node, Vec<&Arc<'found>>> {
        self.arcs.iter().fold(BTreeMap::new(), |mut leaving: BTreeMap<&Node, Vec<&Arc<'found>>>, arc| {
            leaving.entry(AsRef::<Step>::as_ref(arc).as_ref()).or_default().push(arc);
            leaving
        })
    }
}
