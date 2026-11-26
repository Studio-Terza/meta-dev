use design_model::Prefix;
use store_ports::{Columns, Execute, Prepared, Query, StoreFailure};
use transcript_ports::Uuid;

use super::{Prefixes, Shortest};

#[derive(Debug, Columns)]
struct PrefixRow {
    prefix: Prefix,
}

impl Query for Prefixes {
    type Rows = Shortest;

    fn rows(self, statements: &mut impl Execute) -> Result<Shortest, StoreFailure> {
        let found = statements.read_all(self.0.iter().map(PrefixRow::shortest))?;
        let shortest = self
            .0
            .into_iter()
            .zip(found)
            .map(|(message, rows)| PrefixRow::only(rows).map(|prefix| (message, prefix)))
            .collect::<Result<_, StoreFailure>>()?;
        Ok(Shortest(shortest))
    }
}

impl PrefixRow {
    fn shortest(message: &Uuid) -> Prepared<PrefixRow> {
        store_ports::query!(
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
        )
    }

    fn only(rows: Vec<PrefixRow>) -> Result<Prefix, StoreFailure> {
        rows.into_iter().next().map(|row| row.prefix).ok_or(StoreFailure::Missing)
    }
}
