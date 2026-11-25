use crate::Uttered;
use crate::cli::request::command::printed::Lines;
use std::str::FromStr;

use design_model::{JobId, Segment};
use design_runs::{HeldBy, JobRetried, RunsOf};
use grammar_output::Fields;
use grammar_output::field::Integer;
use store_ports::Store;
use structurer::SegmentRef;

use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::output::{Count, Flailing, Line};
use console_ports::Shown;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retry(JobId);

impl FromStr for Retry {
    type Err = Failure;

    fn from_str(word: &str) -> Result<Retry, Failure> {
        let fields = Fields::lexed(word).map_err(|_stray| Failure::Usage)?;
        let job = fields.field::<Integer<_>>().map_err(|_unread| Failure::Usage)?;
        job.map(|number| Retry(JobId::new(number))).ended().map_err(|_unended| Failure::Usage)
    }
}

impl Retry {
    pub fn released(self, store: &impl Store) -> Result<Uttered, Failure> {
        let Retry(job) = self;
        let lock = store.lock()?;
        store.read(RunsOf::from(job))?.first().ok_or(Failure::Unrun(job))?;
        let hold = store.read(HeldBy::from(job))?.ok_or(Failure::Unflailed(job))?;
        store.write(JobRetried::from(&hold))?;
        lock.end()?;
        let owed: Vec<Line> = hold
            .held()
            .map(|flailed| {
                Line::Owed(Flailing::new(SegmentRef::from(AsRef::<Segment>::as_ref(flailed)), job, flailed.run()))
            })
            .collect();
        Ok(Uttered::new(Shown::said(Lines::from(&owed)), Ok(Outcome::Retried { segments: Count::new(owed.len()) })))
    }
}
