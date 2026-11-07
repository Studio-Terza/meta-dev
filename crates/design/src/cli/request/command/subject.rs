mod boundary;
mod placed;
mod shown;

use std::io::Write;

use design_model::{NodeId, Prefix};
use store_ports::Store;
use structurer::SegmentRef;

use super::printed::Printed;
use crate::cli::failure::Failure;
pub use placed::Found;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Node(NodeId),
    Message(Prefix),
    Segment(SegmentRef),
}

impl Subject {
    pub fn shown(self, store: &impl Store, out: &mut impl Write) -> Result<Printed, Failure> {
        match self {
            Subject::Node(id) => Found::find(store, id).and_then(|found| found.printed(store, out)),
            Subject::Message(prefix) => Shown::message(store, &prefix).and_then(|shown| shown.printed(store, out)),
            Subject::Segment(reference) => {
                Shown::segment(store, &reference).and_then(|shown| shown.printed(store, out))
            }
        }
    }
}
