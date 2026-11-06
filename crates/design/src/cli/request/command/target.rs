mod placed;
mod shown;

use std::io::Write;

use design_model::NodeId;
use store_ports::Store;
use transcript_ports::Uuid;

use super::printed::Printed;
use crate::cli::failure::Failure;
use placed::Neighbourhood;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Node(NodeId),
    Message(Uuid),
}

impl From<&str> for Target {
    fn from(word: &str) -> Target {
        word.parse::<NodeId>().map_or_else(|_unnumbered| Target::Message(Uuid::new(word.to_owned())), Target::Node)
    }
}

impl Target {
    pub fn shown(self, store: &impl Store, out: &mut impl Write) -> Result<Printed, Failure> {
        match self {
            Target::Node(id) => Neighbourhood::find(store, id).and_then(|shown| Printed::shown(out, &shown)),
            Target::Message(uuid) => Shown::find(store, &uuid).and_then(|shown| Printed::shown(out, &shown)),
        }
    }
}
