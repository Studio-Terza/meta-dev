use design_jobs::JobState;
use store_ports::{Batch, Execute, Query, StoreFailure, Written};

use super::{JobRepeated, Paid, Remit};
use crate::Refusal;

impl Remit {
    pub(crate) fn paid(&self, statements: &mut impl Execute) -> Result<Paid, Refusal> {
        let covered = self.covered().rows(statements)?;
        self.settled(&covered)
    }
}

impl Paid {
    pub(crate) fn closed(self, state: JobState, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        self.job().map_or(Ok(Written::default()), |job| state.closing(job).written(statements))
    }
}

impl Batch for JobRepeated {
    type Failure = Refusal;

    fn written(self, statements: &mut impl Execute) -> Result<Written, Refusal> {
        Ok(self.0.paid(statements)?.closed(JobState::Repeated, statements)?)
    }
}
