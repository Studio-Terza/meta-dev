use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use design_model::{Ordinal, Segment};
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Version, Written};
use transcript_ports::Uuid;

use crate::boundary::{MessageRow, Rederived, Row, SegmentRow};

#[derive(Debug, Op)]
pub struct Resegmented;

#[derive(Debug)]
struct Reannotated {
    segments: Vec<Segment>,
}

impl Resegmented {
    pub(super) const PENDING: Version = Version::new(20_261_108_000_012);
}

impl Reannotated {
    fn refused(rederived: &[Rederived], annotated: Vec<SegmentRow>) -> Result<(), StoreFailure> {
        let derived: BTreeMap<Segment, &SegmentRow> =
            rederived.iter().flat_map(Rederived::rows).map(|row| (row.segment(), row)).collect();
        let segments: Vec<Segment> = annotated
            .into_iter()
            .filter(|stored| derived.get(&stored.segment()) != Some(&stored))
            .map(|stored| stored.segment())
            .collect();
        segments.is_empty().then_some(()).ok_or_else(|| StoreFailure::Refused(Box::new(Reannotated { segments })))
    }
}

impl fmt::Display for Reannotated {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("re-deriving the stored segments would change these annotated segments:")?;
        self.segments
            .iter()
            .try_for_each(|segment| write!(formatter, " {}#{}", AsRef::<Uuid>::as_ref(segment), Ordinal::from(segment)))
    }
}

impl Error for Reannotated {}

impl Batch for Resegmented {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let messages = MessageRow::every(statements)?;
        let rederived: Vec<Rederived> = messages.iter().map(Rederived::from).collect();
        Reannotated::refused(&rederived, SegmentRow::annotated(statements)?)?;
        let written =
            rederived.iter().map(|row| row.written(statements)).collect::<Result<Vec<Written>, StoreFailure>>()?;
        Ok(Written::total(written.into_iter()))
    }
}
