use std::error::Error;
use std::str::FromStr;

use design_model::{Entered, Entry, Facet, Kin, Node, NodeId, NodeRelation, Project, Provenance, Title};
use newtype::Newtype;
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Written};

use crate::acyclic::{Closing, Refines, Refusal, Supersedes};
use crate::{Link, Relative};

#[derive(Clone, Debug, Op)]
pub struct NewNode(Entry);

#[derive(Clone, Copy, Debug, Op)]
pub struct Linked {
    source: NodeId,
    relation: NodeRelation,
    target: NodeId,
    provenance: Provenance,
}

#[derive(Clone, Copy, Debug)]
struct EdgeColumns {
    source: i64,
    relation: &'static str,
    target: i64,
    provenance: &'static str,
}

#[derive(Debug)]
pub(crate) struct NodeRow {
    id: i64,
    project: String,
    facet: String,
    stage: String,
    provenance: String,
    title: Option<String>,
}

#[derive(Debug)]
pub(crate) struct RelativeRow {
    kin: String,
    id: i64,
    project: String,
    facet: String,
    stage: String,
    provenance: String,
    title: Option<String>,
}

#[derive(Debug)]
pub(crate) struct OutgoingRow {
    relation: String,
    target: i64,
    provenance: String,
}

impl From<Entry> for NewNode {
    fn from(entry: Entry) -> NewNode {
        NewNode(entry)
    }
}

impl Linked {
    #[must_use]
    pub fn new(source: NodeId, relation: NodeRelation, target: NodeId, provenance: Provenance) -> Linked {
        Linked { source, relation, target, provenance }
    }
}

impl From<Linked> for EdgeColumns {
    fn from(linked: Linked) -> EdgeColumns {
        EdgeColumns {
            source: linked.source.into_inner(),
            relation: linked.relation.name(),
            target: linked.target.into_inner(),
            provenance: Entered::from(linked.provenance).name(),
        }
    }
}

impl EdgeColumns {
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO node_edges (source, relation, target, provenance) VALUES (?1, ?2, ?3, ?4)",
            self.source,
            self.relation,
            self.target,
            self.provenance
        ))
    }
}

impl Batch for NewNode {
    type Failure = StoreFailure;

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
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        match self.relation {
            NodeRelation::Refines => Closing::<Refines>::new(self.source, self.target)
                .written(statements, |edged| EdgeColumns::from(self).written(edged)),
            NodeRelation::Supersedes => Closing::<Supersedes>::new(self.source, self.target)
                .written(statements, |edged| EdgeColumns::from(self).written(edged)),
            NodeRelation::Consumes => Ok(EdgeColumns::from(self).written(statements)?),
        }
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
        Ok(row.title.map(Title::new).into_iter().fold(Node::new(NodeId::new(row.id), entry), Node::titled))
    }
}

impl TryFrom<RelativeRow> for Relative {
    type Error = sqlx::Error;

    fn try_from(row: RelativeRow) -> Result<Relative, sqlx::Error> {
        let node = NodeRow {
            id: row.id,
            project: row.project,
            facet: row.facet,
            stage: row.stage,
            provenance: row.provenance,
            title: row.title,
        };
        Ok(Relative::new(NodeRow::decoded(&row.kin)?, Node::try_from(node)?))
    }
}

impl TryFrom<OutgoingRow> for Link {
    type Error = sqlx::Error;

    fn try_from(row: OutgoingRow) -> Result<Link, sqlx::Error> {
        let provenance = Provenance::from(NodeRow::decoded::<Entered>(&row.provenance)?);
        Ok(Link::new(NodeRow::decoded(&row.relation)?, NodeId::new(row.target), provenance))
    }
}

impl NodeRow {
    pub(crate) fn decoded<Column>(text: &str) -> Result<Column, sqlx::Error>
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
            "SELECT id, project, facet, stage, provenance, title FROM nodes WHERE id = ?1",
            id
        ))?;
        Ok(rows.into_iter().map(Node::try_from).collect::<Result<Vec<Node>, sqlx::Error>>()?)
    }

    pub(crate) fn next(statements: &mut Statements<'_>) -> Result<Vec<NodeId>, StoreFailure> {
        let rows = statements.rows(sqlx::query!(r#"SELECT COALESCE(MAX(id), 0) + 1 AS "next!: i64" FROM nodes"#))?;
        Ok(rows.into_iter().map(|row| NodeId::new(row.next)).collect())
    }

    pub(crate) fn proposed(node: &Node, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let id = node.identity().into_inner();
        let entry = node.entry();
        let project = entry.project().to_string();
        let facet = entry.facet().name();
        let stage = entry.stage().name();
        let provenance = Entered::from(entry.provenance()).name();
        let title = node.title().map(ToString::to_string);
        statements.written(sqlx::query!(
            "INSERT INTO nodes (id, project, facet, stage, provenance, title) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            id,
            project,
            facet,
            stage,
            provenance,
            title
        ))
    }
}

impl RelativeRow {
    pub(crate) fn around(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Relative>, StoreFailure> {
        let id = id.into_inner();
        let ancestor = Kin::Ancestor.name();
        let child = Kin::Child.name();
        let rows = statements.rows(sqlx::query_as!(
            RelativeRow,
            r#"SELECT CASE WHEN closure.descendant = ?1 THEN ?2 ELSE ?3 END AS "kin!: String", nodes.id, nodes.project, nodes.facet, nodes.stage, nodes.provenance, nodes.title FROM closure JOIN nodes ON nodes.id = CASE WHEN closure.descendant = ?1 THEN closure.ancestor ELSE closure.descendant END WHERE (closure.descendant = ?1 AND closure.depth > 0) OR (closure.ancestor = ?1 AND closure.depth = 1) ORDER BY CASE WHEN closure.descendant = ?1 THEN -closure.depth ELSE nodes.id END, nodes.id"#,
            id,
            ancestor,
            child
        ))?;
        Ok(rows.into_iter().map(Relative::try_from).collect::<Result<Vec<Relative>, sqlx::Error>>()?)
    }
}

impl OutgoingRow {
    pub(crate) fn from_node(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Link>, StoreFailure> {
        let id = id.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            OutgoingRow,
            "SELECT relation, target, provenance FROM node_edges WHERE source = ?1 ORDER BY relation, target",
            id
        ))?;
        Ok(rows.into_iter().map(Link::try_from).collect::<Result<Vec<Link>, sqlx::Error>>()?)
    }
}
