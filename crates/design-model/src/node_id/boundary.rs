use std::str::FromStr;

use grammar_node::{Ids, Unnumbered};
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(i64);

impl FromStr for NodeId {
    type Err = Unnumbered;

    fn from_str(text: &str) -> Result<NodeId, Unnumbered> {
        Ids::spelled(text).numbered(text).map(NodeId)
    }
}
