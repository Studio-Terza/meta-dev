use design_model::JobId;
use port_core::Op;
use store_ports::{Columns, Execute, Prepared, Query, Staged, StoreFailure};

use super::JobState;
use super::state::job;

#[derive(Clone, Copy, Debug, Op)]
pub struct OpenJobs;

#[derive(Debug, Columns)]
struct OpenJobRow {
    id: JobId,
}

impl Staged for JobState {
    type Key = JobId;

    type State = JobState;

    fn moved(key: &JobId, from: JobState, to: JobState) -> Prepared<()> {
        store_ports::query!(
            r#"
UPDATE
  jobs
SET
  state = ?3
WHERE
  id = ?1
  AND state = ?2
"#,
            key: JobId,
            from: JobState,
            to: JobState
        )
    }
}

impl Query for OpenJobs {
    type Rows = Vec<job::Open>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<job::Open>, StoreFailure> {
        let open = JobState::Open;
        let rows = statements.rows(store_ports::query!(
            OpenJobRow,
            r#"
SELECT
  id
FROM
  jobs
WHERE
  state = ?1
ORDER BY
  id
"#,
            open: JobState
        ))?;
        Ok(rows.into_iter().map(|row| job::Open::held(row.id)).collect())
    }
}
