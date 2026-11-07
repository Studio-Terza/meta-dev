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
    pub fn ordinal(&self) -> Ordinal {
        self.ordinal
    }

    #[must_use]
    pub fn author(&self) -> Authorship {
        self.author
    }

    #[must_use]
    pub fn body(&self) -> &Body {
        &self.body
    }

    #[must_use]
    pub fn sense(&self) -> Option<Sense> {
        self.sense
    }

    #[must_use]
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }
}
