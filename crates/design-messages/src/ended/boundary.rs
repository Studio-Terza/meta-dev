use store_ports::{Columns, Execute, Prepared, Query, StoreFailure};
use transcript_ports::TaskId;

use super::EndedTasks;

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
pub(crate) struct EndedRow {
    task: TaskId,
}

impl EndedRow {
    pub(crate) fn new(task: TaskId) -> EndedRow {
        EndedRow { task }
    }

    pub(crate) fn inserted(&self) -> Prepared<()> {
        let EndedRow { task } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO ended (task)
VALUES
  (?1)
"#,
            task: TaskId
        )
    }
}

impl Query for EndedTasks {
    type Rows = Vec<TaskId>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<TaskId>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            EndedRow,
            r#"
SELECT
  task
FROM
  ended
"#,
        ))?;
        Ok(rows.into_iter().map(|row| row.task).collect())
    }
}
