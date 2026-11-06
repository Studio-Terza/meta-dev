use store_ports::{Query, Statements, StoreFailure, Written};
use transcript_ports::Instant;

use super::{AppliedFile, AppliedId, Earlier, Seen, Unseen};

#[derive(Debug)]
struct EarlierRow {
    id: i64,
    applied_at: String,
}

impl From<EarlierRow> for Earlier {
    fn from(row: EarlierRow) -> Earlier {
        Earlier { id: AppliedId(row.id), at: Instant::new(row.applied_at) }
    }
}

impl Query for AppliedFile {
    type Rows = Seen;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Seen, StoreFailure> {
        let content: &[u8] = self.0.as_ref();
        let rows = statements.rows(sqlx::query_as!(
            EarlierRow,
            "SELECT id, applied_at FROM applied_files WHERE content = ?1",
            content
        ))?;
        Ok(rows.into_iter().next().map_or(Seen::Unseen(Unseen(self.0)), |row| Seen::Before(Earlier::from(row))))
    }
}

impl Unseen {
    pub(crate) fn written(&self, provenance: &str, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let content: &[u8] = self.0.as_ref();
        statements.written(sqlx::query!(
            "INSERT INTO applied_files (content, provenance) VALUES (?1, ?2)",
            content,
            provenance
        ))
    }
}
