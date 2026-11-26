use design_model::{Project, Provenance, Title};
use grammar_notes::Ending;
use store_ports::{Columns, Execute, Query, StoreFailure};

use super::{ImportedEnding, ImportedNotes, Noted};

#[derive(Debug, Columns)]
struct NotedRow {
    provenance: Provenance,
    title: Option<Title>,
}

#[derive(Debug, Columns)]
struct EndedRow {
    ending: Ending,
}

impl EndedRow {
    fn read(project: &Project, statements: &mut impl Execute) -> Result<Option<Ending>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            EndedRow,
            r#"
SELECT
  notes_endings.ending
FROM
  notes_endings
  JOIN nodes ON nodes.id = notes_endings.node
WHERE
  nodes.project = ?1
"#,
            project: Project
        ))?;
        Ok(rows.into_iter().next().map(|row| row.ending))
    }
}

impl NotedRow {
    fn noted(self) -> Option<Noted> {
        let NotedRow { provenance, title } = self;
        provenance.imported().zip(title).map(|(line, heading)| Noted { line, title: heading })
    }
}

impl Query for ImportedNotes {
    type Rows = Vec<Noted>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Noted>, StoreFailure> {
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

impl Query for ImportedEnding {
    type Rows = Option<Ending>;

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Ending>, StoreFailure> {
        EndedRow::read(&self.0, statements)
    }
}
