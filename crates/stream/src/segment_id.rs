use std::fmt;

use transcript_ports::Uuid;

use crate::ordinal::Ordinal;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentId {
    message: Uuid,
    ordinal: Ordinal,
}

impl SegmentId {
    pub(crate) fn new(message: Uuid, ordinal: Ordinal) -> SegmentId {
        SegmentId { message, ordinal }
    }

    #[must_use]
    pub fn message(&self) -> &Uuid {
        &self.message
    }

    #[must_use]
    pub fn ordinal(&self) -> Ordinal {
        self.ordinal
    }
}

impl fmt::Display for SegmentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}#{}", self.message, self.ordinal)
    }
}
