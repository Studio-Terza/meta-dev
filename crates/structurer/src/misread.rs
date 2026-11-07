use std::fmt;

use grammar::directive::{Lexed, Location};
use grammar::output::field::{Integer, Phrase, Word};
use grammar::output::{Fields, Unread};
use named_derive::Named;

use crate::briefing::Quoting;

use crate::verb::Verb;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wanted {
    #[name("a directive")]
    Directive,
    #[name("an opening parenthesis")]
    Open,
    #[name("a comma or a closing parenthesis")]
    Separator,
    #[name("an argument")]
    Argument,
    #[name("the end of the line")]
    End,
    #[name("a segment, as in 85b287b3.0")]
    Segment,
    #[name("a node id, as in n42, or the label of a node proposed earlier in the file")]
    Node,
    #[name("a quoted title")]
    Title,
    #[name("a span relation")]
    SpanRelation,
    #[name("a segment relation")]
    SegmentRelation,
    #[name("a sense kind")]
    SenseKind,
    #[name("a specificity")]
    Specificity,
    #[name("a confidence")]
    Confidence,
    #[name("an explicitness")]
    Explicitness,
    #[name("a facet")]
    Facet,
    #[name("new, or new and a label that is not a node id")]
    New,
    #[name("refines(<node>)")]
    Refines,
    #[name("by(<node>)")]
    By,
    #[name("an end other than the first, as nothing relates to itself")]
    Distinct,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Misread {
    #[error("line {}, column {}: expected {wanted}", at.line(), at.column())]
    Expected { at: Location, wanted: Wanted },
    #[error("line {}, column {}: {verb} is written {}", at.line(), at.column(), verb.form())]
    Arity { at: Location, verb: Verb },
}

impl Misread {
    pub(crate) fn expected(lexed: &Lexed, wanted: Wanted) -> Misread {
        Misread::Expected { at: Location::from(lexed), wanted }
    }
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Missing {
    #[name("expected")]
    Expected,
    #[name("arity")]
    Arity,
}

pub(crate) struct Briefed<'misread>(&'misread Misread);

impl fmt::Display for Briefed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Misread::Expected { at, wanted } => {
                write!(formatter, "{} {} {} {}", Missing::Expected, at.line(), at.column(), Quoting::from(wanted))
            }
            Misread::Arity { at, verb } => write!(formatter, "{} {} {} {verb}", Missing::Arity, at.line(), at.column()),
        }
    }
}

impl Misread {
    pub(crate) fn read(fields: &mut Fields) -> Result<Misread, Unread> {
        let missing = fields.field::<Word<Missing>>()?;
        let at = Location::new(fields.field::<Integer<_>>()?, fields.field::<Integer<_>>()?);
        missing.misread(at, fields)
    }
}

impl Missing {
    fn misread(self, at: Location, fields: &mut Fields) -> Result<Misread, Unread> {
        match self {
            Missing::Expected => fields.field::<Phrase<_>>().map(|wanted| Misread::Expected { at, wanted }),
            Missing::Arity => fields.field::<Word<_>>().map(|verb| Misread::Arity { at, verb }),
        }
    }
}

impl<'misread> From<&'misread Misread> for Briefed<'misread> {
    fn from(misread: &'misread Misread) -> Briefed<'misread> {
        Briefed(misread)
    }
}
