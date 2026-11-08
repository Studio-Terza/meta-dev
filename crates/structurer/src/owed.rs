use std::collections::BTreeSet;
use std::fmt;
use std::iter;

use grammar::cited::Cited;
use grammar::output::Fields;

use crate::SegmentRef;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Owed(BTreeSet<SegmentRef>);

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the reply gives no directive to segment {first} and {} more of its job's segments", rest.len())]
pub struct Uncovered {
    first: SegmentRef,
    rest: Vec<SegmentRef>,
}

pub(crate) struct Listed<'uncovered>(&'uncovered Uncovered);

impl Owed {
    pub fn paid(&self, covering: &BTreeSet<SegmentRef>) -> Result<(), Uncovered> {
        let left: Vec<SegmentRef> = self.0.difference(covering).cloned().collect();
        left.split_first()
            .map(|(first, rest)| Uncovered { first: first.clone(), rest: rest.to_vec() })
            .map_or(Ok(()), Err)
    }
}

impl FromIterator<SegmentRef> for Owed {
    fn from_iter<Segments: IntoIterator<Item = SegmentRef>>(segments: Segments) -> Owed {
        Owed(segments.into_iter().collect())
    }
}

impl From<&mut Fields> for Owed {
    fn from(fields: &mut Fields) -> Owed {
        iter::from_fn(|| fields.optional::<Cited>()).map(|cited| SegmentRef::from(&cited)).collect()
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
