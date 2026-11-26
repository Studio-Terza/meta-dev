use std::collections::BTreeSet;

use design_model::{Ordinal, Segment};
use store_ports::{Columns, Execute, Prepared, Query, StoreFailure};
use transcript_ports::Uuid;

use super::Covered;

#[derive(Debug, Columns)]
struct CoveredRow {
    message: Uuid,
    ordinal: Ordinal,
}

impl Covered {
    fn of_message(message: &Uuid) -> Prepared<CoveredRow> {
        store_ports::query!(
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
        )
    }
}

impl Query for Covered {
    type Rows = BTreeSet<Segment>;

    fn rows(self, statements: &mut impl Execute) -> Result<BTreeSet<Segment>, StoreFailure> {
        let covered = statements.read_all(self.0.iter().map(Covered::of_message))?;
        Ok(covered.into_iter().flatten().map(|row| Segment::new(row.message, row.ordinal)).collect())
    }
}
