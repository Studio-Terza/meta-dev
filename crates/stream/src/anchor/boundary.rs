use std::str::FromStr;

use grammar::anchor::Anchoring;
use transcript_ports::Uuid;

use super::Anchor;
use crate::ordinal::Ordinal;
use crate::segment_id::SegmentId;

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is neither a message id nor one followed by # and a segment index")]
pub struct Unanchored(String);

impl Anchor {
    fn anchored(message: Uuid, ordinal: Option<usize>) -> Anchor {
        match ordinal {
            None => Anchor::Message(message),
            Some(ordinal) => Anchor::Segment(SegmentId::new(message, Ordinal::new(ordinal))),
        }
    }
}

impl FromStr for Anchor {
    type Err = Unanchored;

    fn from_str(written: &str) -> Result<Anchor, Unanchored> {
        Anchoring::read(written)
            .and_then(|anchoring| {
                anchoring.message().parse::<Uuid>().ok().map(|message| Anchor::anchored(message, anchoring.ordinal()))
            })
            .ok_or_else(|| Unanchored(written.to_owned()))
    }
}
