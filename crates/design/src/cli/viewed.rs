use std::fmt;

use design_model::{NodeId, Outline, Provenance, SpanRelation};
use grammar_cited::Cited;
use grammar_output::field::{Id, Quoted, Word};
use grammar_output::{Fields, Unread};
use named_derive::Named;
use newtype::Newtype;
use structurer::{Outlined, SegmentRef};
use transcript_ports::{Authorship, Body};

use crate::cli::output::Key;

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
pub(crate) enum Seen {
    #[name("outline")]
    Outline,
    #[name("standing")]
    Standing,
    #[name("unplaced")]
    Unplaced,
}

impl Viewed {
    pub(crate) fn read(seen: Seen, fields: &mut Fields) -> Result<Viewed, Unread> {
        match seen {
            Seen::Outline => Outlined::read(fields).map(Viewed::Outlined),
            Seen::Standing => Ok(Viewed::Standing {
                node: fields.field::<Id<_>>()?,
                relation: fields.field::<Word<_>>()?,
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                author: fields.field::<Word<_>>()?,
                entered: fields.past(&[Key::Entered])?.field::<Word<_>>()?,
                body: fields.field::<Quoted<_>>()?,
            }),
            Seen::Unplaced => Ok(Viewed::Unplaced {
                segment: SegmentRef::from(&fields.field::<Cited>()?),
                body: fields.field::<Quoted<_>>()?,
            }),
        }
    }
}

impl fmt::Display for Viewed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Viewed::Outlined(outline) => write!(formatter, "{} {}", Seen::Outline, Outlined::from(outline)),
            Viewed::Standing { node, relation, segment, author, entered, body } => write!(
                formatter,
                "{} n{} {relation} {segment} {author} {} {entered} {}",
                Seen::Standing,
                node.into_inner(),
                Key::Entered,
                Fields::quoting(body.as_ref())
            ),
            Viewed::Unplaced { segment, body } => {
                write!(formatter, "{} {segment} {}", Seen::Unplaced, Fields::quoting(body.as_ref()))
            }
        }
    }
}
