use design_jobs::JobState;
use store_ports::{Batch, Done, Execute, Query, StoreFailure};

use super::{JobRepeated, Paid, Remit};
use crate::Refusal;

impl Remit {
    pub(crate) fn paid(&self, statements: &mut impl Execute) -> Result<Paid, Refusal> {
        let covered = self.covered().rows(statements)?;
        self.settled(&covered)
    }
}

impl Paid {
    pub(crate) fn closed<D: Done>(self, state: JobState, statements: &mut impl Execute) -> Result<D, StoreFailure> {
        self.job().map_or(Ok(D::unchanged()), |job| state.closing(job).moved(statements))
    }
}

impl Batch for JobRepeated {
    type Failure = Refusal;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), Refusal> {
        Ok(self.0.paid(statements)?.closed(JobState::Repeated, statements)?)
    }
}
