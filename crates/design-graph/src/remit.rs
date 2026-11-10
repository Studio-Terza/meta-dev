use std::collections::BTreeSet;

use design_model::{JobId, Segment};
use port_core::Op;
use store_ports::{Batch, Query, Statements, StoreFailure, Written};
use transcript_ports::Uuid;

use design_jobs::{JobClosed, JobState};

use crate::{Covered, Refusal, Unpaid};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remit {
    Unbounded,
    Job { job: JobId, segments: BTreeSet<Segment> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Paid {
    Unbounded,
    Job(JobId),
}

#[derive(Debug, Op)]
pub struct JobRepeated(Remit);

impl Remit {
    #[must_use]
    pub fn job(job: JobId, segments: impl IntoIterator<Item = Segment>) -> Remit {
        Remit::Job { job, segments: segments.into_iter().collect() }
    }

    #[must_use]
    pub fn admitted(&self, segment: Segment) -> Option<Segment> {
        Some(segment).filter(|candidate| self.segments().is_none_or(|segments| segments.contains(candidate)))
    }

    fn covered(&self) -> Covered {
        self.segments().into_iter().flatten().map(|segment| AsRef::<Uuid>::as_ref(segment).clone()).collect()
    }

    pub(crate) fn paid(&self, statements: &mut Statements<'_>) -> Result<Paid, Refusal> {
        let covered = self.covered().rows(statements)?;
        self.settled(&covered)
    }

    fn settled(&self, covered: &BTreeSet<Segment>) -> Result<Paid, Refusal> {
        match self {
            Remit::Unbounded => Ok(Paid::Unbounded),
            Remit::Job { job, segments } => {
                Paid::owing(*job, &segments.difference(covered).cloned().collect::<Vec<_>>())
            }
        }
    }

    fn segments(&self) -> Option<&BTreeSet<Segment>> {
        match self {
            Remit::Unbounded => None,
            Remit::Job { segments, .. } => Some(segments),
        }
    }
}

impl Paid {
    fn owing(job: JobId, left: &[Segment]) -> Result<Paid, Refusal> {
        left.split_first()
            .map_or(Ok(Paid::Job(job)), |(first, rest)| Err(Refusal::Unpaid(Unpaid::new(first.clone(), rest.to_vec()))))
    }

    pub(crate) fn closed(self, state: JobState, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Paid::Unbounded => Ok(Written::default()),
            Paid::Job(job) => JobClosed::new(job, state).written(statements),
        }
    }
}

impl From<Remit> for JobRepeated {
    fn from(remit: Remit) -> JobRepeated {
        JobRepeated(remit)
    }
}

impl Batch for JobRepeated {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        Ok(self.0.paid(statements)?.closed(JobState::Repeated, statements)?)
    }
}
