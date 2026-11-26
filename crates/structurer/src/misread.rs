mod boundary;

use std::fmt;

use grammar_directive::{Lexed, Location};
use grammar_output::field::{Phrase, Word};
use grammar_output::{Fields, Mapped, Parsed, Unread};
use newtype::Listed;
use strum::{Display, IntoStaticStr, VariantArray};

use crate::briefing::Quoting;

use crate::verb::Verb;

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wanted {
    #[strum(serialize = "a directive")]
    Directive,
    #[strum(serialize = "an opening parenthesis")]
    Open,
    #[strum(serialize = "a comma or a closing parenthesis")]
    Separator,
    #[strum(serialize = "an argument")]
    Argument,
    #[strum(serialize = "the end of the line")]
    End,
    #[strum(serialize = "a segment, as in 85b287b3.0")]
    Segment,
    #[strum(serialize = "a node id, as in n42, or the label of a node proposed earlier in the file")]
    Node,
    #[strum(serialize = "a quoted title")]
    Title,
    #[strum(serialize = "a span relation")]
    SpanRelation,
    #[strum(serialize = "a segment relation")]
    SegmentRelation,
    #[strum(serialize = "a sense kind")]
    SenseKind,
    #[strum(serialize = "a specificity")]
    Specificity,
    #[strum(serialize = "a confidence")]
    Confidence,
    #[strum(serialize = "an explicitness")]
    Explicitness,
    #[strum(serialize = "a facet")]
    Facet,
    #[strum(serialize = "new, or new and a label that is not a node id")]
    New,
    #[strum(serialize = "refines(<node>)")]
    Refines,
    #[strum(serialize = "by(<node>)")]
    By,
    #[strum(serialize = "an end other than the first, as nothing relates to itself")]
    Distinct,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Misread {
    #[error("line {}, column {}: expected {wanted}", at.as_ref(), at.column())]
    Expected { at: Location, wanted: Wanted },
    #[error("line {}, column {}: {verb} is written {}", at.as_ref(), at.column(), verb.form())]
    Arity { at: Location, verb: Verb },
}

impl Misread {
    pub(crate) fn expected(lexed: &Lexed, wanted: Wanted) -> Misread {
        Misread::Expected { at: Location::from(lexed), wanted }
    }
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Missing {
    Expected,
    Arity,
}

pub(crate) struct Briefed<'misread>(&'misread Misread);

impl fmt::Display for Briefed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Misread::Expected { at, wanted } => {
                write!(formatter, "{} {} {} {}", Missing::Expected, at.as_ref(), at.column(), Quoting::from(wanted))
            }
            Misread::Arity { at, verb } => {
                write!(formatter, "{} {} {} {verb}", Missing::Arity, at.as_ref(), at.column())
            }
        }
    }
}

impl Missing {
    fn misread(self, at: Location, fields: Fields) -> Result<Parsed<Misread, Fields>, Unread> {
        match self {
            Missing::Expected => fields.field::<Phrase<_>>().mapped(|wanted| Misread::Expected { at, wanted }),
            Missing::Arity => fields.field::<Word<_>>().mapped(|verb| Misread::Arity { at, verb }),
        }
    }
}

impl<'misread> From<&'misread Misread> for Briefed<'misread> {
    fn from(misread: &'misread Misread) -> Briefed<'misread> {
        Briefed(misread)
    }
}
