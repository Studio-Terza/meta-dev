use design_model::{NodeId, Ordinal, Segment};
use store_ports::{Query, Statements, StoreFailure};
use transcript_ports::{Authorship, Uuid};

use super::{NextNode, Prefixed, Spoken};
use crate::graph::NodeRow;

#[derive(Debug)]
struct SpokenRow {
    uuid: String,
    author: String,
    ordinal: i64,
}

impl TryFrom<SpokenRow> for Spoken {
    type Error = sqlx::Error;

    fn try_from(row: SpokenRow) -> Result<Spoken, sqlx::Error> {
        let author: Authorship = row.author.parse().map_err(|unvoiced| sqlx::Error::Decode(Box::new(unvoiced)))?;
        let ordinal = usize::try_from(row.ordinal).map_err(|negative| sqlx::Error::Decode(Box::new(negative)))?;
        Ok(Spoken { segment: Segment::new(Uuid::new(row.uuid), Ordinal::new(ordinal)), author })
    }
}

impl Query for Prefixed {
    type Rows = Vec<Spoken>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Spoken>, StoreFailure> {
        let prefix = self.0.to_string();
        let rows = statements.rows(sqlx::query_as!(
            SpokenRow,
            "SELECT messages.uuid, messages.author, segments.ordinal FROM messages JOIN segments ON segments.message = messages.uuid WHERE substr(messages.uuid, 1, length(?1)) = ?1 ORDER BY messages.uuid, segments.ordinal",
            prefix
        ))?;
        Ok(rows.into_iter().map(Spoken::try_from).collect::<Result<Vec<Spoken>, sqlx::Error>>()?)
    }
}

impl Query for NextNode {
    type Rows = NodeId;

    fn rows(self, statements: &mut Statements<'_>) -> Result<NodeId, StoreFailure> {
        Ok(NodeRow::next(statements)?.into_iter().next().unwrap_or(NodeId::new(1)))
    }
}
