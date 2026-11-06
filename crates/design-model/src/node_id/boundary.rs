use std::str::FromStr;

use grammar::node::{Ids, Spelled};
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(i64);

#[derive(Debug, thiserror::Error)]
pub enum Unnumbered {
    #[error("{0:?} is not a node id: its number has a leading zero or is too large for a node")]
    Misnumbered(String),
    #[error("{0:?} is not a node id, which is n followed by its number, as in n42")]
    Unspelled(String),
}

impl Unnumbered {
    pub fn unspelled(self) -> Result<String, Unnumbered> {
        match self {
            Unnumbered::Unspelled(text) => Ok(text),
            misnumbered @ Unnumbered::Misnumbered(_) => Err(misnumbered),
        }
    }
}

impl FromStr for NodeId {
    type Err = Unnumbered;

    fn from_str(text: &str) -> Result<NodeId, Unnumbered> {
        match Ids::spelled(text) {
            Spelled::Numbered(number) => Ok(NodeId(number)),
            Spelled::Misnumbered => Err(Unnumbered::Misnumbered(text.to_owned())),
            Spelled::Unspelled => Err(Unnumbered::Unspelled(text.to_owned())),
        }
    }
}
