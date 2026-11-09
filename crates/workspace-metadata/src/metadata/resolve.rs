mod node;

use serde::Deserialize;

pub use node::{Node, NodeDependency};

#[derive(Clone, Debug, Deserialize)]
pub struct Resolve {
    nodes: Vec<Node>,
}

impl Resolve {
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}
