use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure};
use transcript_ports::Body;

use super::{Overran, Overruns};

#[derive(Clone, Debug, PartialEq, Eq, Columns)]
struct OverrunRow {
    task: Body,
}

impl OverrunRow {
    fn inserted(&self) -> Prepared<()> {
        let OverrunRow { task } = self;
        store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO overruns (task)
VALUES
  (?1)
"#,
            task: Body
        )
    }
}

impl Batch for Overran {
    type Failure = StoreFailure;

    type Done = ();

    fn written(self, statements: &mut impl Execute) -> Result<(), StoreFailure> {
        let Overran(tasks) = self;
        let rows: Vec<OverrunRow> = tasks.into_iter().map(|task| OverrunRow { task }).collect();
        statements.written_all(rows.iter().map(OverrunRow::inserted))
    }
}

impl Query for Overruns {
    type Rows = Vec<Body>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Body>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            OverrunRow,
            r#"
SELECT
  task
FROM
  overruns
"#,
        ))?;
        Ok(rows.into_iter().map(|row| row.task).collect())
    }
}
