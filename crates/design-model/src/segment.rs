use newtype::{Arbitrary, Display, Newtype, Wrapping};
use transcript_ports::Uuid;

use crate::Prefix;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ordinal(usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Segment {
    message: Uuid,
    ordinal: Ordinal,
}

impl Segment {
    #[must_use]
    pub fn new(message: Uuid, ordinal: Ordinal) -> Segment {
        Segment { message, ordinal }
    }
}

impl AsRef<Uuid> for Segment {
    fn as_ref(&self) -> &Uuid {
        &self.message
    }
}

impl From<&Segment> for Ordinal {
    fn from(segment: &Segment) -> Ordinal {
        segment.ordinal
    }
}

impl From<&Segment> for Prefix {
    fn from(segment: &Segment) -> Prefix {
        Prefix::new(segment.message.clone().into_inner())
    }
}
