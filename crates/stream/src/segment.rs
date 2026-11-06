use grammar::message::Span;
use transcript_ports::Body;

use crate::{Form, Message, SegmentId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub id: SegmentId,
    pub range: Span,
    pub form: Form,
}

impl Segment {
    #[must_use]
    pub fn text(&self, message: &Message) -> Body {
        Body::new(self.range.slice(message.record().text.as_ref()).to_owned())
    }
}
