use std::fmt;

use design_store::{RelationName, ShapeName, ShownMessage, Stored};
use store_ports::Store;
use transcript_ports::{Authorship, Body, Instant, SessionId, Uuid};

use crate::cli::failure::Failure;
use crate::cli::rendering::Annotated;

#[derive(Debug)]
pub struct Shown(Stored);

impl Shown {
    pub fn find(store: &impl Store, uuid: &Uuid) -> Result<Shown, Failure> {
        store.read(ShownMessage::named(uuid.clone()))?.map(Shown).ok_or_else(|| Failure::Missing(uuid.clone()))
    }
}

impl fmt::Display for Shown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self.0.message();
        let uuid: &Uuid = message.as_ref();
        let author: &Authorship = message.as_ref();
        let session: &SessionId = message.as_ref();
        let stamp: &Instant = message.as_ref();
        writeln!(formatter, "{uuid} {author} session {session} at {stamp}")?;
        self.0.segments().iter().try_for_each(|segment| {
            let shape: &ShapeName = segment.as_ref();
            writeln!(formatter, "  {} {shape}: {:?}", segment.ordinal(), Body::from(segment).to_string())
        })?;
        self.0.links().iter().try_for_each(|link| {
            let [source, target] = link.anchors();
            let relation: &RelationName = link.as_ref();
            writeln!(formatter, "  link {source} {relation} {target}")
        })?;
        self.0.annotations().iter().try_for_each(|annotation| writeln!(formatter, "  {}", Annotated::from(annotation)))
    }
}
