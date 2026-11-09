use std::io::Write;
use std::str::FromStr;

use design_model::{JobId, Segment};
use design_store::{Flailed, FlailedSegments, JobRetried, RunsOf};
use grammar::output::Fields;
use grammar::output::field::Integer;
use store_ports::Store;
use structurer::SegmentRef;

use super::Printed;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::output::{Count, Flailing, Line};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retry(JobId);

impl FromStr for Retry {
    type Err = Failure;

    fn from_str(word: &str) -> Result<Retry, Failure> {
        let mut fields = Fields::lexed(word).map_err(|_stray| Failure::Usage)?;
        let job = fields.field::<Integer<_>>().map(JobId::new).map_err(|_unread| Failure::Usage)?;
        fields.ended(Retry(job)).map_err(|_unended| Failure::Usage)
    }
}

impl Retry {
    pub fn released(self, store: &impl Store, out: &mut impl Write) -> Result<Outcome, Failure> {
        let Retry(job) = self;
        let lock = store.lock()?;
        store.read(RunsOf::from(job))?.first().ok_or(Failure::Unrun(job))?;
        let held: Vec<Flailed> =
            store.read(FlailedSegments)?.into_iter().filter(|flailed| flailed.job() == job).collect();
        held.first().ok_or(Failure::Unflailed(job))?;
        store.write(JobRetried::from(job))?;
        lock.end()?;
        let owed: Vec<Line> = held
            .iter()
            .map(|flailed| {
                Line::Owed(Flailing::new(SegmentRef::from(AsRef::<Segment>::as_ref(flailed)), job, flailed.run()))
            })
            .collect();
        Printed::lines(out, &owed)?;
        Ok(Outcome::Retried { segments: Count::new(owed.len()) })
    }
}
