use std::cmp::Ordering;

use design_model::{NodeId, Stage, Title};
use grammar_search::Term;
use newtype::{Arbitrary, Newtype};
use store_ports::{Columns, Statements, StoreFailure};

use super::Recalled;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq)]
pub(super) struct Score(f64);

#[derive(Debug, Columns)]
pub(super) struct RecallRow {
    node: NodeId,
    stage: Stage,
    title: Title,
    rank: Score,
}

impl RecallRow {
    pub(super) fn matching(term: &Term, statements: &mut Statements<'_>) -> Result<Vec<RecallRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            RecallRow,
            r#"
SELECT
  nodes.id AS "node!: i64",
  nodes.stage AS "stage!: String",
  nodes.title AS "title!: String",
  hits.rank AS "rank!: f64"
FROM
  (
    SELECT
      rowid,
      rank
    FROM
      segments_fts
    WHERE
      segments_fts MATCH ?1
    ORDER BY
      rank
    LIMIT
      200
  ) AS hits
  JOIN segments ON segments.rowid = hits.rowid
  JOIN span_edges ON span_edges.message = segments.message
  AND span_edges.ordinal = segments.ordinal
  JOIN nodes ON nodes.id = span_edges.node
  JOIN live_nodes ON live_nodes.id = nodes.id
WHERE
  nodes.title <> ''
"#,
            term: Term
        ))
    }

    pub(super) fn recalled(self) -> Recalled {
        let RecallRow { node, stage, title, rank } = self;
        Recalled { node, stage, title, score: Score(-rank.0) }
    }
}

impl Score {
    pub(super) fn plus(self, other: Score) -> Score {
        Score(self.0 + other.0)
    }

    pub(super) fn ordered(self, other: Score) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
