use design_model::{Ordinal, Segment};
use port_core::Op;
use store_ports::{Columns, Query, Statements, StoreFailure};
use transcript_ports::{Body, Uuid};

#[derive(Clone, Debug, Op)]
pub struct AskedEarlier(Uuid);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    segment: Segment,
    body: Body,
}

#[derive(Debug, Columns)]
struct AskedRow {
    message: Uuid,
    ordinal: Ordinal,
    body: Body,
}

impl From<&Segment> for AskedEarlier {
    fn from(segment: &Segment) -> AskedEarlier {
        AskedEarlier(AsRef::<Uuid>::as_ref(segment).clone())
    }
}

impl From<AskedRow> for Asked {
    fn from(row: AskedRow) -> Asked {
        Asked { segment: Segment::new(row.message, row.ordinal), body: row.body }
    }
}

impl AsRef<Segment> for Asked {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}

impl AsRef<Body> for Asked {
    fn as_ref(&self) -> &Body {
        &self.body
    }
}

impl Query for AskedEarlier {
    type Rows = Vec<Asked>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Asked>, StoreFailure> {
        let job = self.0;
        let rows = statements.rows(store_ports::query!(
            AskedRow,
            r#"
SELECT
  latest.message AS "message!: String",
  latest.ordinal AS "ordinal!: i64",
  latest.body AS "body!: String"
FROM
  (
    SELECT
      segments.message,
      segments.ordinal,
      segments.body,
      asked.at
    FROM
      open_questions
      JOIN messages AS asked ON asked.uuid = open_questions.message
      JOIN messages AS job ON job.uuid = ?1
      JOIN segments ON segments.message = open_questions.message
      AND segments.ordinal = open_questions.ordinal
    WHERE
      asked.session = job.session
      AND asked.at < job.at
    GROUP BY
      segments.message,
      segments.ordinal
    ORDER BY
      asked.at DESC,
      segments.message DESC,
      segments.ordinal DESC
    LIMIT
      8
  ) AS latest
ORDER BY
  latest.at,
  latest.message,
  latest.ordinal
"#,
            job: Uuid
        ))?;
        Ok(rows.into_iter().map(Asked::from).collect())
    }
}
