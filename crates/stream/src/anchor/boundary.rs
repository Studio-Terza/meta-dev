use std::str::FromStr;

use grammar_anchor::Anchoring;
use transcript_ports::Uuid;

use super::Anchor;
use crate::ignored::Ignored;
use crate::ordinal::Ordinal;
use crate::segment_id::SegmentId;
use invariants::Ignoring;

#[derive(Debug, thiserror::Error)]
#[error("{0:?} is neither a message id nor one followed by # and a segment index")]
pub struct Unanchored(String);

impl Anchor {
    fn anchored(message: Uuid, ordinal: Option<u32>) -> Anchor {
        match ordinal {
            None => Anchor::Message(message),
            Some(counted) => Anchor::Segment(SegmentId::new(message, Ordinal::new(counted))),
        }
    }
}

impl FromStr for Anchor {
    type Err = Unanchored;

    fn from_str(written: &str) -> Result<Anchor, Unanchored> {
        written
            .parse::<Anchoring>()
            .ignoring(Ignored::NotAnAnchor)
            .and_then(|anchoring| {
                let message = Uuid::try_new(anchoring.message().to_owned()).ignoring(Ignored::NotAnAnchor)?;
                Some(Anchor::anchored(message, anchoring.ordinal()))
            })
            .ok_or_else(|| Unanchored(written.to_owned()))
    }
}
