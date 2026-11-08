use grammar::message::Span;
use transcript_ports::{Body, Nul};

use crate::Message;
use crate::form::Form;
use crate::segment_id::SegmentId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    id: SegmentId,
    range: Span,
    form: Form,
}

impl Segment {
    pub(crate) fn new(id: SegmentId, range: Span, form: Form) -> Segment {
        Segment { id, range, form }
    }

    #[must_use]
    pub fn identity(&self) -> &SegmentId {
        &self.id
    }

    #[must_use]
    pub fn form(&self) -> Form {
        self.form
    }

    pub fn text(&self, message: &Message) -> Result<Body, Nul> {
        Body::try_new(self.range.slice(message.record().text().as_ref()).to_owned())
    }
}
