use std::collections::BTreeSet;

use design_model::Segment;
use transcript_ports::Uuid;

use crate::Covered;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remit {
    Unbounded,
    Job(BTreeSet<Segment>),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the directives leave segments of their job unstructured")]
pub struct Unpaid {
    first: Segment,
    rest: Vec<Segment>,
}

impl Remit {
    #[must_use]
    pub fn admitted(&self, segment: Segment) -> Option<Segment> {
        Some(segment).filter(|segment| self.job().is_none_or(|job| job.contains(segment)))
    }

    #[must_use]
    pub fn covered(&self) -> Covered {
        self.job().into_iter().flatten().map(|segment| AsRef::<Uuid>::as_ref(segment).clone()).collect()
    }

    pub fn paid(&self, covered: &BTreeSet<Segment>) -> Result<(), Unpaid> {
        let left: Vec<Segment> = self.job().into_iter().flat_map(|job| job.difference(covered)).cloned().collect();
        left.split_first().map(|(first, rest)| Unpaid { first: first.clone(), rest: rest.to_vec() }).map_or(Ok(()), Err)
    }

    fn job(&self) -> Option<&BTreeSet<Segment>> {
        match self {
            Remit::Unbounded => None,
            Remit::Job(segments) => Some(segments),
        }
    }
}

impl FromIterator<Segment> for Remit {
    fn from_iter<Segments: IntoIterator<Item = Segment>>(segments: Segments) -> Remit {
        Remit::Job(segments.into_iter().collect())
    }
}

impl Unpaid {
    #[must_use]
    pub fn first(&self) -> &Segment {
        &self.first
    }

    #[must_use]
    pub fn rest(&self) -> &[Segment] {
        &self.rest
    }
}
