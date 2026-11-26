use std::collections::BTreeSet;

use design_model::{Ordinal, Segment};
use store_ports::{Columns, Execute, Query, StoreFailure};
use transcript_ports::Uuid;

use super::Covered;

#[derive(Debug, Columns)]
struct CoveredRow {
    message: Uuid,
    ordinal: Ordinal,
}

impl Covered {
    fn of_message(message: &Uuid, statements: &mut impl Execute) -> Result<Vec<Segment>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            CoveredRow,
            r#"
SELECT DISTINCT
  message AS "message!: String",
  ordinal AS "ordinal!: i64"
FROM
  structured
WHERE
  message = ?1
ORDER BY
  ordinal
"#,
            message: Uuid
        ))?;
        Ok(rows.into_iter().map(|row| Segment::new(row.message, row.ordinal)).collect())
    }
}

impl Query for Covered {
    type Rows = BTreeSet<Segment>;

    fn rows(self, statements: &mut impl Execute) -> Result<BTreeSet<Segment>, StoreFailure> {
        let covered = self
            .0
            .iter()
            .map(|message| Covered::of_message(message, statements))
            .collect::<Result<Vec<Vec<Segment>>, StoreFailure>>()?;
        Ok(covered.into_iter().flatten().collect())
    }
}
