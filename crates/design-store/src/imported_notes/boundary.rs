use design_model::{Project, Provenance, Title};
use store_ports::{Columns, Query, Statements, StoreFailure};

use super::{ImportedNotes, Noted};

#[derive(Debug, Columns)]
struct NotedRow {
    provenance: Provenance,
    title: Option<Title>,
}

impl NotedRow {
    fn noted(self) -> Option<Noted> {
        let NotedRow { provenance, title } = self;
        provenance.imported().zip(title).map(|(line, title)| Noted { line, title })
    }
}

impl Query for ImportedNotes {
    type Rows = Vec<Noted>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Noted>, StoreFailure> {
        let project: Project = self.0;
        let rows = statements.rows(store_ports::query!(
            NotedRow,
            r#"
SELECT
  provenance,
  title
FROM
  nodes
WHERE
  project = ?1
ORDER BY
  id
"#,
            project: Project
        ))?;
        Ok(rows.into_iter().filter_map(NotedRow::noted).collect())
    }
}
