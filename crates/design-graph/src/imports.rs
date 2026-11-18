mod boundary;

use design_model::{Node, NodeId, Provenance, Vague};
use grammar_notes::Ending;
use port_core::Op;

use crate::Directed;

#[derive(Debug, Op)]
pub struct Imports {
    ending: Ending,
    lines: Vec<Imported>,
}

#[derive(Debug)]
pub struct Imported {
    provenance: Provenance,
    node: Node<Vague>,
    parent: NodeId,
    directed: Vec<Directed>,
}

impl Imported {
    #[must_use]
    pub fn new(provenance: Provenance, node: Node<Vague>, parent: NodeId, directed: Vec<Directed>) -> Imported {
        Imported { provenance, node, parent, directed }
    }
}

impl Imports {
    #[must_use]
    pub fn new(ending: Ending, lines: Vec<Imported>) -> Imports {
        Imports { ending, lines }
    }
}
