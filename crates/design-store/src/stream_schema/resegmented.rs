use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use design_model::{Ordinal, Segment};
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Version, Written};
use transcript_ports::Uuid;

use crate::boundary::{MessageRow, Row, SegmentRow};

#[derive(Debug, Op)]
pub struct Resegmented;

#[derive(Debug)]
struct Referenced {
    segments: Vec<Segment>,
}

impl Resegmented {
    pub(super) const PENDING: Version = Version::new(20_261_108_000_012);

    fn stale(derived: &[SegmentRow], stored: Vec<SegmentRow>) -> Vec<SegmentRow> {
        let derived: BTreeMap<Segment, &SegmentRow> = derived.iter().map(|row| (row.segment(), row)).collect();
        stored.into_iter().filter(|row| derived.get(&row.segment()) != Some(&row)).collect()
    }
}

impl Referenced {
    fn refused(released: Vec<(Segment, Option<Written>)>) -> Result<Written, StoreFailure> {
        let segments: Vec<Segment> =
            released.iter().filter(|(_, written)| written.is_none()).map(|(segment, _)| segment.clone()).collect();
        segments
            .is_empty()
            .then(|| Written::total(released.into_iter().filter_map(|(_, written)| written)))
            .ok_or_else(|| StoreFailure::Refused(Box::new(Referenced { segments })))
    }
}

impl fmt::Display for Referenced {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("re-deriving the stored segments would change these segments that stored rows refer to:")?;
        self.segments
            .iter()
            .try_for_each(|segment| write!(formatter, " {}#{}", AsRef::<Uuid>::as_ref(segment), Ordinal::from(segment)))
    }
}

impl Error for Referenced {}

impl Batch for Resegmented {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let messages = MessageRow::every(statements)?;
        let derived: Vec<SegmentRow> = messages.iter().flat_map(SegmentRow::within).collect();
        let released = Resegmented::stale(&derived, SegmentRow::every(statements)?)
            .iter()
            .map(|row| row.released(statements).map(|written| (row.segment(), written)))
            .collect::<Result<Vec<_>, StoreFailure>>()?;
        let dropped = Referenced::refused(released)?;
        let written =
            derived.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter().chain([dropped])))
    }
}
