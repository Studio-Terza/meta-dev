mod boundary;

use design_model::Segment;
use port_core::Op;
use transcript_ports::Body;

#[derive(Clone, Debug, Op)]
pub struct AskedEarlier<'segment>(&'segment Segment);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    segment: Segment,
    body: Body,
}

impl<'segment> From<&'segment Segment> for AskedEarlier<'segment> {
    fn from(segment: &'segment Segment) -> AskedEarlier<'segment> {
        AskedEarlier(segment)
    }
}

impl AsRef<Segment> for Asked {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}

impl AsRef<Body> for Asked {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}
