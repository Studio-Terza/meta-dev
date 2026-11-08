mod boundary;

use design_model::{NodeId, Ordinal, Provenance, Segment, SpanRelation};
use port_core::Op;
use transcript_ports::{Authorship, Body, Uuid};

#[derive(Clone, Copy, Debug, Op)]
pub struct CurrentSpans(SpanRelation);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Uttered {
    node: NodeId,
    relation: SpanRelation,
    message: Uuid,
    ordinal: Ordinal,
    author: Authorship,
    provenance: Provenance,
    body: Body,
}

impl From<SpanRelation> for CurrentSpans {
    fn from(relation: SpanRelation) -> CurrentSpans {
        CurrentSpans(relation)
    }
}

impl Uttered {
    #[must_use]
    pub fn node(&self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn relation(&self) -> SpanRelation {
        self.relation
    }

    #[must_use]
    pub fn segment(&self) -> Segment {
        Segment::new(self.message.clone(), self.ordinal)
    }
}

impl AsRef<Authorship> for Uttered {
    fn as_ref(&self) -> &Authorship {
        &self.author
    }
}

impl AsRef<Provenance> for Uttered {
    fn as_ref(&self) -> &Provenance {
        &self.provenance
    }
}

impl AsRef<Body> for Uttered {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}
