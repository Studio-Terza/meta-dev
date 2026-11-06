use std::fmt;

use design_store::{ShownMessage, Stored};
use store_ports::Store;
use transcript_ports::{Body, Uuid};

use crate::cli::failure::Failure;

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
        writeln!(
            formatter,
            "{} {} session {} at {}",
            message.uuid(),
            message.author(),
            message.session(),
            message.stamp()
        )?;
        self.0.segments().iter().try_for_each(|segment| {
            writeln!(formatter, "  {} {}: {:?}", segment.ordinal(), segment.shape(), Body::from(segment).to_string())
        })?;
        self.0.links().iter().try_for_each(|link| {
            let [source, target] = link.anchors();
            writeln!(formatter, "  link {source} {} {target}", link.relation())
        })
    }
}
