use std::error::Error;
use std::str::FromStr;

use design_model::{Entered, Entry, Facet, Node, NodeId, NodeRelation, Project, Provenance};
use newtype::Newtype;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Written};

use crate::Link;

#[derive(Clone, Debug, Op)]
pub struct NewNode(Entry);

#[derive(Clone, Copy, Debug, Op)]
pub struct Linked {
    source: NodeId,
    relation: NodeRelation,
    target: NodeId,
}

#[derive(Debug)]
pub(crate) struct NodeRow {
    id: i64,
    project: String,
    facet: String,
    stage: String,
    provenance: String,
}

#[derive(Debug)]
pub(crate) struct OutgoingRow {
    relation: String,
    target: i64,
}

impl From<Entry> for NewNode {
    fn from(entry: Entry) -> NewNode {
        NewNode(entry)
    }
}

impl Linked {
    #[must_use]
    pub fn new(source: NodeId, relation: NodeRelation, target: NodeId) -> Linked {
        Linked { source, relation, target }
    }
}

impl Batch for NewNode {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let entry = self.0;
        let project = entry.project().to_string();
        let facet = entry.facet().name();
        let stage = entry.stage().name();
        let provenance = Entered::from(entry.provenance()).name();
        statements.written(sqlx::query!(
            "INSERT INTO nodes (project, facet, stage, provenance) VALUES (?1, ?2, ?3, ?4)",
            project,
            facet,
            stage,
            provenance
        ))
    }
}

impl Batch for Linked {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let source = self.source.into_inner();
        let relation = self.relation.name();
        let target = self.target.into_inner();
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO node_edges (source, relation, target) VALUES (?1, ?2, ?3)",
            source,
            relation,
            target
        ))
    }
}

impl TryFrom<NodeRow> for Node {
    type Error = sqlx::Error;

    fn try_from(row: NodeRow) -> Result<Node, sqlx::Error> {
        let entry = Entry::new(
            NodeRow::decoded(&row.project)?,
            NodeRow::decoded(&row.facet)?,
            NodeRow::decoded(&row.stage)?,
            Provenance::from(NodeRow::decoded::<Entered>(&row.provenance)?),
        );
        Ok(Node::new(NodeId::new(row.id), entry))
    }
}

impl TryFrom<OutgoingRow> for Link {
    type Error = sqlx::Error;

    fn try_from(row: OutgoingRow) -> Result<Link, sqlx::Error> {
        Ok(Link::new(NodeRow::decoded(&row.relation)?, NodeId::new(row.target)))
    }
}

impl NodeRow {
    fn decoded<Column>(text: &str) -> Result<Column, sqlx::Error>
    where
        Column: FromStr<Err: Error + Send + Sync + 'static>,
    {
        text.parse().map_err(|unknown| sqlx::Error::Decode(Box::new(unknown)))
    }

    pub(crate) fn root(project: &Project, statements: &mut Statements<'_>) -> Result<Vec<NodeId>, StoreFailure> {
        let project = project.to_string();
        let facet = Facet::Project.name();
        let rows = statements.rows(sqlx::query!(
            "SELECT id FROM nodes WHERE project = ?1 AND facet = ?2 ORDER BY id LIMIT 1",
            project,
            facet
        ))?;
        Ok(rows.into_iter().map(|row| NodeId::new(row.id)).collect())
    }

    pub(crate) fn named(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Node>, StoreFailure> {
        let id = id.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            NodeRow,
            "SELECT id, project, facet, stage, provenance FROM nodes WHERE id = ?1",
            id
        ))?;
        Ok(rows.into_iter().map(Node::try_from).collect::<Result<Vec<Node>, sqlx::Error>>()?)
    }
}

impl OutgoingRow {
    pub(crate) fn from_node(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Link>, StoreFailure> {
        let id = id.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            OutgoingRow,
            "SELECT relation, target FROM node_edges WHERE source = ?1 ORDER BY relation, target",
            id
        ))?;
        Ok(rows.into_iter().map(Link::try_from).collect::<Result<Vec<Link>, sqlx::Error>>()?)
    }
}
