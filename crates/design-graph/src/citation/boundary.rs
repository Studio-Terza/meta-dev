use design_model::{NodeId, Provenance};
use store_ports::{Columns, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::Citation;

#[derive(Debug, Columns)]
struct CitationRow {
    message: Uuid,
    author: Authorship,
    provenance: Provenance,
}

impl Citation {
    pub(crate) fn cited_by(node: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Citation>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            CitationRow,
            r#"
SELECT
  node_cites.message,
  messages.author,
  node_cites.provenance
FROM
  node_cites
  JOIN messages ON messages.uuid = node_cites.message
WHERE
  node_cites.node = ?1
ORDER BY
  node_cites.message
"#,
            node: NodeId
        ))?;
        Ok(rows
            .into_iter()
            .map(|row| Citation { message: row.message, author: row.author, provenance: row.provenance })
            .collect())
    }
}
