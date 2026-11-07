mod boundary;

use design_model::{Ordinal, Provenance, Segment, Sense, SpanRelation};
use transcript_ports::{Authorship, Body, Uuid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    relation: SpanRelation,
    message: Uuid,
    ordinal: Ordinal,
    author: Authorship,
    body: Body,
    sense: Option<Sense>,
    provenance: Provenance,
}

impl Placement {
    #[must_use]
    pub fn relation(&self) -> SpanRelation {
        self.relation
    }

    #[must_use]
    pub fn segment(&self) -> Segment {
        Segment::new(self.message.clone(), self.ordinal)
    }

    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    #[must_use]
    pub fn sense(&self) -> Option<Sense> {
        self.sense
    }
}

impl AsRef<Authorship> for Placement {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}

impl AsRef<Provenance> for Placement {
    fn as_ref(&self) -> &Provenance {
        &self.provenance
    }
}
