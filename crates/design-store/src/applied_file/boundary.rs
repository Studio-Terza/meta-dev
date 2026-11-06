use design_model::Entered;
use file_ports::Bytes;
use store_ports::{Columns, Query, Statements, StoreFailure, Written};
use transcript_ports::Instant;

use super::{AppliedFile, AppliedId, Earlier, Seen, Unseen};

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

    fn rows(self, statements: &mut Statements<'_>) -> Result<Seen, StoreFailure> {
        let content = &self.0;
        let rows = statements.rows(store_ports::query!(
            EarlierRow,
            "SELECT id, applied_at FROM applied_files WHERE content = ?1",
            content: Bytes
        ))?;
        Ok(rows.into_iter().next().map_or(Seen::Unseen(Unseen(self.0)), |row| Seen::Before(Earlier::from(row))))
    }
}

impl Unseen {
    pub(crate) fn written(&self, provenance: Entered, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let content = &self.0;
        statements.written(store_ports::query!(
            "INSERT INTO applied_files (content, provenance) VALUES (?1, ?2)",
            content: Bytes,
            provenance: Entered
        ))
    }
}
