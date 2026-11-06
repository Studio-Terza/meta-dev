mod placed;
mod shown;

use std::io::Write;

use design_model::NodeId;
use store_ports::Store;
use transcript_ports::Uuid;

use super::printed::Printed;
use crate::cli::failure::Failure;
use crate::cli::rendering::Neighbourhood;
pub use placed::Found;
use shown::Shown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
    Node(NodeId),
    Message(Uuid),
}

impl TryFrom<&str> for Subject {
    type Error = Failure;

    fn try_from(word: &str) -> Result<Subject, Failure> {
        word.parse::<NodeId>()
            .map(Subject::Node)
            .or_else(|unnumbered| Ok(Subject::Message(Uuid::new(unnumbered.unspelled()?))))
    }
}

impl Subject {
    pub fn shown(self, store: &impl Store, out: &mut impl Write) -> Result<Printed, Failure> {
        match self {
            Subject::Node(id) => {
                Found::find(store, id).and_then(|found| Printed::shown(out, &Neighbourhood::from(found.placed())))
            }
            Subject::Message(uuid) => Shown::find(store, &uuid).and_then(|shown| Printed::shown(out, &shown)),
        }
    }
}
