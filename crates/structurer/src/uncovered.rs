use std::fmt;
use std::iter;

use grammar::cited::Cited;
use grammar::output::{Fields, Unread};

use crate::SegmentRef;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the reply gives no directive to segment {first} and {} more of its job's segments", rest.len())]
pub struct Uncovered {
    first: SegmentRef,
    rest: Vec<SegmentRef>,
}

pub(crate) struct Listed<'uncovered>(&'uncovered Uncovered);

impl Uncovered {
    #[must_use]
    pub fn new(first: SegmentRef, rest: Vec<SegmentRef>) -> Uncovered {
        Uncovered { first, rest }
    }
}

impl TryFrom<&mut Fields> for Uncovered {
    type Error = Unread;

    fn try_from(fields: &mut Fields) -> Result<Uncovered, Unread> {
        let first = SegmentRef::from(&fields.field::<Cited>()?);
        let rest = iter::from_fn(|| fields.optional::<Cited>()).map(|cited| SegmentRef::from(&cited));
        Ok(Uncovered::new(first, rest.collect()))
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
