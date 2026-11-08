mod node;

use serde::Deserialize;

pub use node::{Node, NodeDependency};

#[derive(Debug, Deserialize)]
pub struct Resolve {
    nodes: Vec<Node>,
}

impl AsRef<[Node]> for Resolve {
    fn as_ref(&self) -> &[Node] {
        &self.nodes
    }
}
