use std::str::FromStr;

use grammar::node::Ids;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(i64);

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is not a node id, which is n followed by its number, as in n42")]
pub struct Unnumbered(String);

impl FromStr for NodeId {
    type Err = Unnumbered;

    fn from_str(text: &str) -> Result<NodeId, Unnumbered> {
        Ids::numbered(text).map(NodeId).ok_or_else(|| Unnumbered(text.to_owned()))
    }
}
