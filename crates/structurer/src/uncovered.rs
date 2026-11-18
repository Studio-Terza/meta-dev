use std::fmt;
use std::iter;

use grammar_cited::Cited;
use grammar_output::Record;

use crate::SegmentRef;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Record)]
#[error("the reply gives no directive to segment {first} and {} more of its job's segments", rest.len())]
pub struct Uncovered {
    #[field(Cited, via = SegmentRef::from)]
    first: SegmentRef,
    #[repeated(Cited, via = SegmentRef::from)]
    rest: Vec<SegmentRef>,
}

pub(crate) struct Listed<'uncovered>(&'uncovered Uncovered);

impl Uncovered {
    #[must_use]
    pub fn new(first: SegmentRef, rest: Vec<SegmentRef>) -> Uncovered {
        Uncovered { first, rest }
    }
}

impl fmt::Display for Listed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        iter::once(&self.0.first).chain(&self.0.rest).try_for_each(|segment| write!(formatter, " {segment}"))
    }
}

impl<'uncovered> From<&'uncovered Uncovered> for Listed<'uncovered> {
    fn from(uncovered: &'uncovered Uncovered) -> Listed<'uncovered> {
        Listed(uncovered)
    }
}
