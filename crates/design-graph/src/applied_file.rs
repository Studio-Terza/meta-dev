mod boundary;

use file_ports::Bytes;
use newtype::{Arbitrary, Newtype};
use port_core::Op;
use transcript_ports::Instant;

#[derive(Clone, Debug, Op)]
pub struct AppliedFile(Bytes);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppliedId(i64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Earlier {
    id: AppliedId,
    at: Instant,
}

#[derive(Debug)]
pub struct Unseen(Bytes);

pub trait Sees {
    type Output;

    fn before(self, earlier: Earlier) -> Self::Output;

    fn unseen(self, unseen: Unseen) -> Self::Output;
}

#[derive(Debug)]
pub enum Seen {
    Before(Earlier),
    Unseen(Unseen),
}

impl From<Bytes> for AppliedFile {
    fn from(contents: Bytes) -> AppliedFile {
        AppliedFile(contents)
    }
}

impl Earlier {
    #[must_use]
    pub fn identity(&self) -> AppliedId {
        self.id
    }

    #[must_use]
    pub fn stamp(&self) -> &Instant {
        &self.at
    }
}

impl Seen {
    pub fn seen<Visitor: Sees>(self, visitor: Visitor) -> Visitor::Output {
        match self {
            Seen::Before(earlier) => visitor.before(earlier),
            Seen::Unseen(unseen) => visitor.unseen(unseen),
        }
    }
}

impl AsRef<Bytes> for Unseen {
    fn as_ref(&self) -> &Bytes {
        &self.0
    }
}
