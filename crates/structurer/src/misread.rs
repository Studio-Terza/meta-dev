use grammar::directive::{Lexed, Location};
use named_derive::Named;

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
    #[name("a node id, as in n42")]
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
    #[name("new")]
    New,
    #[name("refines(<node>)")]
    Refines,
    #[name("by(<node>)")]
    By,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
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
