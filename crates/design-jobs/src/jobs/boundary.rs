use design_model::JobId;
use store_ports::{Batch, Statements, StoreFailure, Written};

use super::{JobClosed, JobState};

impl Batch for JobClosed {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let JobClosed { job, state } = self;
        statements.written(store_ports::query!(
            r#"
UPDATE
  jobs
SET
  state = ?2
WHERE
  id = ?1
"#,
            job: JobId,
            state: JobState
        ))
    }
}
