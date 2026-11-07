mod boundary;

use design_model::{Ordinal, Prefix, Provenance, Sense, SpanRelation};
use transcript_ports::{Authorship, Body};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    relation: SpanRelation,
    prefix: Prefix,
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
    pub fn prefix(&self) -> &Prefix {
        &self.prefix
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

impl From<&Placement> for Ordinal {
    fn from(placement: &Placement) -> Ordinal {
        placement.ordinal
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
