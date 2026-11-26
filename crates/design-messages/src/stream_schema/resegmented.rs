mod boundary;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use design_model::{Ordinal, Segment};
use port_core::Op;
use store_ports::{Cause, Released, StoreFailure, Version};
use transcript_ports::Uuid;

use crate::boundary::SegmentRow;

#[derive(Debug, Op)]
pub struct Resegmented;

#[derive(Debug)]
struct Referenced {
    segments: Vec<Segment>,
}

impl Resegmented {
    pub(super) const PENDING: Version = Version::new(20_261_108_000_012);

    fn stale(derived: &[SegmentRow], stored: Vec<SegmentRow>) -> Vec<SegmentRow> {
        let keyed: BTreeMap<Segment, &SegmentRow> = derived.iter().map(|row| (row.segment(), row)).collect();
        stored.into_iter().filter(|row| keyed.get(&row.segment()) != Some(&row)).collect()
    }
}

impl Referenced {
    fn kept(segments: Vec<Segment>, derived: Vec<SegmentRow>) -> Result<Vec<SegmentRow>, StoreFailure> {
        match segments.first() {
            None => Ok(derived),
            Some(_) => Err(StoreFailure::Refused(Cause::told(&Referenced { segments }))),
        }
    }

    fn cleared(
        derived: Vec<SegmentRow>,
        stale: &[SegmentRow],
        released: &[Released],
    ) -> Result<Vec<SegmentRow>, StoreFailure> {
        let segments: Vec<Segment> = stale
            .iter()
            .zip(released)
            .filter(|(_, release)| **release == Released::Referenced)
            .map(|(row, _)| row.segment())
            .collect();
        Referenced::kept(segments, derived)
    }
}

impl fmt::Display for Referenced {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .write_str("re-deriving the stored segments would change these segments that stored rows refer to:")?;
        self.segments
            .iter()
            .try_for_each(|segment| write!(formatter, " {}#{}", AsRef::<Uuid>::as_ref(segment), Ordinal::from(segment)))
    }
}

impl Error for Referenced {}
