use design_model::Prefix;
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::Uuid;

use super::{Prefixes, Shortest};

#[derive(Debug, Columns)]
struct PrefixRow {
    prefix: Prefix,
}

impl Query for Prefixes {
    type Rows = Shortest;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Shortest, StoreFailure> {
        let shortest = self
            .0
            .into_iter()
            .map(|message| PrefixRow::shortest(&message, statements).map(|prefix| (message, prefix)))
            .collect::<Result<_, StoreFailure>>()?;
        Ok(Shortest(shortest))
    }
}

impl PrefixRow {
    fn shortest(message: &Uuid, statements: &mut Statements<'_>) -> Result<Prefix, StoreFailure> {
        let row = statements.one(store_ports::query!(
            PrefixRow,
            r#"
WITH
RECURSIVE neighbours (uuid) AS (
  SELECT
    max(uuid)
  FROM
    messages
  WHERE
    uuid < ?1
  UNION ALL
  SELECT
    min(uuid)
  FROM
    messages
  WHERE
    uuid > ?1
),
lengths (n) AS (
  SELECT
    8
  UNION ALL
  SELECT
    n + 1
  FROM
    lengths
  WHERE
    n < length(?1)
)
SELECT
  substr(?1, 1, coalesce(min(n), length(?1))) AS "prefix!: String"
FROM
  lengths
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      neighbours
    WHERE
      substr(neighbours.uuid, 1, n) = substr(?1, 1, n)
  )
"#,
            message: Uuid
        ))?;
        Ok(row.prefix)
    }
}
