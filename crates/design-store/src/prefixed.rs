mod boundary;

use design_model::{Prefix, Segment};
use port_core::Op;
use transcript_ports::Authorship;

#[derive(Clone, Debug, Op)]
pub struct Prefixed(Prefix);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spoken {
    segment: Segment,
    author: Authorship,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct NextNode;

impl From<Prefix> for Prefixed {
    fn from(prefix: Prefix) -> Prefixed {
        Prefixed(prefix)
    }
}

impl Spoken {
    #[must_use]
    pub fn segment(&self) -> &Segment {
        &self.segment
    }

    #[must_use]
    pub fn author(&self) -> Authorship {
        self.author
    }
}
