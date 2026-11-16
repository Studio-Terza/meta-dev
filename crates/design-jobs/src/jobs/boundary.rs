use design_model::JobId;
use store_ports::{Prepared, Staged};

use super::JobState;

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
