use std::io::Write;

use design_model::{Ordinal, Prefix};
use design_store::{ShownMessage, Started, Stored};
use newtype::Newtype;
use store_ports::Store;
use structurer::SegmentRef;
use transcript_ports::Uuid;

use super::super::printed::Printed;
use crate::cli::failure::{Failure, Unmatched};
use crate::cli::request::Unresolved;
use crate::cli::request::command::rendering::Refs;

#[derive(Debug)]
pub struct Shown(Stored);

impl Shown {
    pub fn message(store: &impl Store, prefix: &Prefix) -> Result<Shown, Failure> {
        let exact = store.read(ShownMessage::named(Uuid::new(prefix.clone().into_inner())))?;
        let stored = exact.map_or_else(|| Shown::prefixed(store, prefix), Ok)?;
        Ok(Shown(stored))
    }

    pub fn segment(store: &impl Store, reference: &SegmentRef) -> Result<Shown, Failure> {
        let prefix: &Prefix = reference.as_ref();
        let shown = Shown::message(store, prefix)?;
        let wanted = Ordinal::from(reference);
        shown
            .0
            .segments()
            .iter()
            .any(|segment| segment.ordinal() == wanted)
            .then_some(shown)
            .ok_or_else(|| Failure::from(Unresolved::Unsegmented(reference.clone())))
    }

    pub fn printed(&self, store: &impl Store, out: &mut impl Write) -> Result<Printed, Failure> {
        Printed::lines(out, &Refs::stored(store, &self.0)?)
    }

    fn prefixed(store: &impl Store, prefix: &Prefix) -> Result<Stored, Failure> {
        let messages = store.read(Started::from(prefix.clone()))?;
        let [message] = <[Uuid; 1]>::try_from(messages).map_err(|found| {
            found
                .first()
                .map_or_else(|| Unmatched::NoMatch(prefix.clone()), |_several| Unmatched::Ambiguous(prefix.clone()))
        })?;
        let stored = store.read(ShownMessage::named(message))?;
        stored.ok_or_else(|| Failure::from(Unmatched::NoMatch(prefix.clone())))
    }
}
