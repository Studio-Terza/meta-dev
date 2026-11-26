use design_model::Provenance;
use file_ports::Bytes;
use store_ports::Record as _;
use store_ports::{Columns, Done, Execute, Query, StoreFailure};
use transcript_ports::Instant;

use super::{AppliedFile, AppliedId, Earlier, Seen, Unseen};

#[derive(Clone, Debug, PartialEq, Eq, Columns, store_ports::Record)]
#[record(table = "applied_files")]
pub struct AppliedFileRow {
    #[record(generated_key)]
    id: AppliedId,
    content: Bytes,
    #[record(generated)]
    applied_at: Instant,
    provenance: Provenance,
}

#[derive(Debug, Columns)]
struct EarlierRow {
    id: AppliedId,
    applied_at: Instant,
}

impl From<EarlierRow> for Earlier {
    fn from(row: EarlierRow) -> Earlier {
        Earlier { id: row.id, at: row.applied_at }
    }
}

impl Query for AppliedFile {
    type Rows = Seen;

    fn rows(self, statements: &mut impl Execute) -> Result<Seen, StoreFailure> {
        let content = &self.0;
        let rows = statements.rows(store_ports::query!(
            EarlierRow,
            r#"
SELECT
  id,
  applied_at
FROM
  applied_files
WHERE
  content = ?1
"#,
            content: Bytes
        ))?;
        Ok(rows.into_iter().next().map_or(Seen::Unseen(Unseen(self.0)), |row| Seen::Before(Earlier::from(row))))
    }
}

impl Unseen {
    pub(crate) fn written<D: Done>(
        &self,
        provenance: Provenance,
        statements: &mut impl Execute,
    ) -> Result<D, StoreFailure> {
        let fresh = AppliedFileRowNew { content: self.0.clone(), provenance };
        statements.one(AppliedFileRow::created(&fresh)).map(|_stored| D::changed(1))
    }
}
