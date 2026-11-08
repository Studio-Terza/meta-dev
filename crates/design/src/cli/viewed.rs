use std::fmt;

use design_model::{NodeId, Outline, Provenance, SpanRelation};
use grammar::cited::Cited;
use grammar::output::field::{Id, Quoted, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;
use newtype::Newtype;
use structurer::{Outlined, SegmentRef};
use transcript_ports::{Authorship, Body};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Viewed {
    Outlined(Outline),
    Standing {
        node: NodeId,
        relation: SpanRelation,
        segment: SegmentRef,
        author: Authorship,
        entered: Provenance,
        body: Body,
    },
    Unplaced {
        segment: SegmentRef,
        body: Body,
    },
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Looked {
    #[name("outline")]
    Outline,
    #[name("standing")]
    Standing,
    #[name("unplaced")]
    Unplaced,
    #[name("entered")]
    Entered,
}

impl Looked {
    pub(crate) fn viewed(self, fields: &mut Fields) -> Result<Viewed, Unread> {
        match self {
            Looked::Outline => Outlined::read(fields).map(Viewed::Outlined),
            Looked::Standing => Ok(Viewed::Standing {
                node: fields.field::<Id<_>>()?,
                relation: fields.field::<Word<_>>()?,
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                author: fields.field::<Word<_>>()?,
                entered: fields.past(&[Looked::Entered])?.field::<Word<_>>()?,
                body: fields.field::<Quoted<_>>()?,
            }),
            Looked::Unplaced => Ok(Viewed::Unplaced {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                body: fields.field::<Quoted<_>>()?,
            }),
            Looked::Entered => Err(Unread::Unkeyed),
        }
    }
}

impl fmt::Display for Viewed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Viewed::Outlined(outline) => write!(formatter, "{} {}", Looked::Outline, Outlined::from(outline)),
            Viewed::Standing { node, relation, segment, author, entered, body } => write!(
                formatter,
                "{} n{} {relation} {segment} {author} {} {entered} {}",
                Looked::Standing,
                node.into_inner(),
                Looked::Entered,
                Fields::quoting(body.as_ref())
            ),
            Viewed::Unplaced { segment, body } => {
                write!(formatter, "{} {segment} {}", Looked::Unplaced, Fields::quoting(body.as_ref()))
            }
        }
    }
}
