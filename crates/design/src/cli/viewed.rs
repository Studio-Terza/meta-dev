use std::fmt;

use design_model::{NodeId, Outline, Provenance, SpanRelation};
use grammar_cited::Cited;
use grammar_output::field::{Id, Quoted, Word};
use grammar_output::{Fields, Mapped, Parsed, Record, Unread};
use newtype::{Listed, Wrapping};
use structurer::{Outlined, SegmentRef};
use strum::{Display, IntoStaticStr, VariantArray};
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

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub(crate) enum Seen {
    Outline,
    Standing,
    Unplaced,
}

#[derive(Debug, Record)]
struct Standing {
    #[field(Id<_>)]
    node: NodeId,
    #[field(Word<_>)]
    relation: SpanRelation,
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Word<_>)]
    author: Authorship,
    #[past(Key::Entered)]
    #[field(Word<_>)]
    entered: Provenance,
    #[field(Quoted<_>)]
    body: Body,
}

#[derive(Debug, Record)]
struct Unplaced {
    #[field(Cited, via = SegmentRef::from)]
    segment: SegmentRef,
    #[field(Quoted<_>)]
    body: Body,
}

impl Seen {
    pub(crate) fn viewed(self, fields: Fields) -> Result<Parsed<Viewed, Fields>, Unread> {
        match self {
            Seen::Outline => Outlined::read(fields).mapped(Viewed::Outlined),
            Seen::Standing => fields.record::<Standing>().map(Parsed::converted),
            Seen::Unplaced => fields.record::<Unplaced>().map(Parsed::converted),
        }
    }
}

impl From<Standing> for Viewed {
    fn from(read: Standing) -> Viewed {
        let Standing { node, relation, segment, author, entered, body } = read;
        Viewed::Standing { node, relation, segment, author, entered, body }
    }
}

impl From<Unplaced> for Viewed {
    fn from(read: Unplaced) -> Viewed {
        Viewed::Unplaced { segment: read.segment, body: read.body }
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
