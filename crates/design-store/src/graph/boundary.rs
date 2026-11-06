use std::error::Error;
use std::str::FromStr;

use design_model::{
    AnyNode, AnyStage, Dropped, Entered, Entry, Facet, Kin, Node, NodeEdge, NodeId, NodeRelation, Project, Provenance,
    Reason, Refining, Role, Stage, Staging, Terminal, Title, Vague,
};
use newtype::{Counter, Newtype};
use port_core::Op;
use store_ports::{Batch, Statements, StoreFailure, Written};

use crate::acyclic::{Acyclic, Closing, Refines, Refusal, Supersedes};
use crate::{Link, Relative};

#[derive(Clone, Debug, Op)]
pub struct Planting {
    sown: Option<Node<Vague>>,
    linked: Option<Linked>,
}

#[derive(Clone, Debug, Op)]
pub struct Restaged(AnyNode);

#[derive(Clone, Copy, Debug, Op)]
pub struct Cast {
    node: NodeId,
    role: Role,
}

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
    role: String,
    artifact: Option<String>,
    condition: Option<String>,
    reason: Option<String>,
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
    role: String,
    artifact: Option<String>,
    condition: Option<String>,
    reason: Option<String>,
}

#[derive(Debug)]
struct StageColumns {
    stage: String,
    artifact: Option<String>,
    condition: Option<String>,
    reason: Option<String>,
}

#[derive(Debug)]
struct LeaningRow {
    source: i64,
    relation: String,
    target: i64,
    source_role: String,
    target_role: String,
}

#[derive(Debug)]
pub(crate) struct OutgoingRow {
    relation: String,
    target: i64,
    provenance: String,
}

impl Planting {
    #[must_use]
    pub fn new(sown: Option<Node<Vague>>, linked: Option<Linked>) -> Planting {
        Planting { sown, linked }
    }
}

impl From<AnyNode> for Restaged {
    fn from(node: AnyNode) -> Restaged {
        Restaged(node)
    }
}

impl Cast {
    #[must_use]
    pub fn new(node: NodeId, role: Role) -> Cast {
        Cast { node, role }
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
    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let written = statements.written(sqlx::query!(
            "INSERT OR IGNORE INTO node_edges (source, relation, target, provenance) VALUES (?1, ?2, ?3, ?4)",
            self.source,
            self.relation,
            self.target,
            self.provenance
        ))?;
        LeaningRow::upheld(NodeId::new(self.source), statements).map(|_upheld| written)
    }
}

impl Batch for Planting {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let sown = self
            .sown
            .iter()
            .map(|node| NodeRow::proposed(node, statements))
            .collect::<Result<Vec<Written>, StoreFailure>>()?;
        let linked = self
            .linked
            .into_iter()
            .map(|linked| linked.written(statements))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Ok(Written::total(sown.into_iter().chain(linked)))
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
            NodeRelation::Consumes => EdgeColumns::from(self).written(statements),
        }
    }
}

impl Batch for Restaged {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let id = self.0.identity().into_inner();
        let StageColumns { stage, artifact, condition, reason } = StageColumns::from(self.0.stage());
        statements.written(sqlx::query!(
            "UPDATE nodes SET stage = ?2, artifact = ?3, condition = ?4, reason = ?5 WHERE id = ?1",
            id,
            stage,
            artifact,
            condition,
            reason
        ))
    }
}

impl Batch for Cast {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let id = self.node.into_inner();
        let role = self.role.name();
        let written = statements.written(sqlx::query!("UPDATE nodes SET role = ?2 WHERE id = ?1", id, role))?;
        LeaningRow::upheld(self.node, statements).map(|_upheld| written)
    }
}

impl From<&AnyStage> for StageColumns {
    fn from(stage: &AnyStage) -> StageColumns {
        stage.staged(StageColumns::bare(stage.stage()))
    }
}

impl StageColumns {
    fn bare(stage: Stage) -> StageColumns {
        StageColumns { stage: stage.name().to_owned(), artifact: None, condition: None, reason: None }
    }
}

impl Staging for StageColumns {
    type Output = StageColumns;

    fn vague(self, _vague: &Vague) -> StageColumns {
        self
    }

    fn refining(self, _refining: &Refining) -> StageColumns {
        self
    }

    fn terminal(self, terminal: &Terminal) -> StageColumns {
        StageColumns {
            artifact: Some(terminal.artifact().to_string()),
            condition: Some(terminal.condition().to_string()),
            ..self
        }
    }

    fn dropped(self, dropped: &Dropped) -> StageColumns {
        StageColumns { reason: Some(dropped.reason().to_string()), ..self }
    }
}

impl TryFrom<StageColumns> for AnyStage {
    type Error = sqlx::Error;

    fn try_from(columns: StageColumns) -> Result<AnyStage, sqlx::Error> {
        match NodeRow::decoded::<Stage>(&columns.stage)? {
            Stage::Vague => Ok(AnyStage::Vague(Vague)),
            Stage::Refining => Ok(AnyStage::Refining(Refining)),
            Stage::Terminal => Ok(AnyStage::Terminal(Terminal::new(
                NodeRow::decoded(columns.artifact.as_deref().unwrap_or_default())?,
                NodeRow::decoded(columns.condition.as_deref().unwrap_or_default())?,
            ))),
            Stage::Dropped => Ok(AnyStage::Dropped(Dropped::from(NodeRow::decoded::<Reason>(
                columns.reason.as_deref().unwrap_or_default(),
            )?))),
        }
    }
}

impl TryFrom<NodeRow> for AnyNode {
    type Error = sqlx::Error;

    fn try_from(row: NodeRow) -> Result<AnyNode, sqlx::Error> {
        let entry = Entry::new(
            NodeRow::decoded(&row.project)?,
            NodeRow::decoded(&row.facet)?,
            Provenance::from(NodeRow::decoded::<Entered>(&row.provenance)?),
        );
        let columns =
            StageColumns { stage: row.stage, artifact: row.artifact, condition: row.condition, reason: row.reason };
        let node = AnyNode::loaded(NodeId::new(row.id), entry, NodeRow::decoded(&row.role)?, columns.try_into()?);
        Ok(row.title.map(Title::new).into_iter().fold(node, Node::titled))
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
            role: row.role,
            artifact: row.artifact,
            condition: row.condition,
            reason: row.reason,
        };
        Ok(Relative::new(NodeRow::decoded(&row.kin)?, AnyNode::try_from(node)?))
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

    pub(crate) fn named(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<AnyNode>, StoreFailure> {
        let id = id.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            NodeRow,
            "SELECT id, project, facet, stage, provenance, title, role, artifact, condition, reason FROM nodes WHERE id = ?1",
            id
        ))?;
        Ok(rows.into_iter().map(AnyNode::try_from).collect::<Result<Vec<AnyNode>, sqlx::Error>>()?)
    }

    pub(crate) fn next(statements: &mut Statements<'_>) -> Result<Vec<NodeId>, StoreFailure> {
        let rows = statements.rows(sqlx::query!(r#"SELECT COALESCE(MAX(id), 0) + 1 AS "next!: i64" FROM nodes"#))?;
        Ok(rows.into_iter().map(|row| NodeId::new(row.next)).collect())
    }

    pub(crate) fn proposed(node: &Node<Vague>, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let id = node.identity().into_inner();
        let entry = node.entry();
        let project = entry.project().to_string();
        let facet = entry.facet().name();
        let stage = Stage::Vague.name();
        let provenance = Entered::from(entry.provenance()).name();
        let title = node.title().map(ToString::to_string);
        let role = node.role().name();
        statements.written(sqlx::query!(
            "INSERT INTO nodes (id, project, facet, stage, provenance, title, role) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            id,
            project,
            facet,
            stage,
            provenance,
            title,
            role
        ))
    }
}

impl RelativeRow {
    pub(crate) fn around<R: Acyclic<End = NodeId>>(
        id: NodeId,
        statements: &mut Statements<'_>,
    ) -> Result<Vec<Relative>, StoreFailure> {
        let id = id.into_inner();
        let ancestor = Kin::Ancestor.name();
        let child = Kin::Child.name();
        let relation = R::CLOSED.name();
        let rows = statements.rows(sqlx::query_as!(
            RelativeRow,
            r#"SELECT CASE WHEN closure.descendant = ?1 THEN ?2 ELSE ?3 END AS "kin!: String", nodes.id, nodes.project, nodes.facet, nodes.stage, nodes.provenance, nodes.title, nodes.role, nodes.artifact, nodes.condition, nodes.reason FROM closure JOIN nodes ON nodes.id = CASE WHEN closure.descendant = ?1 THEN closure.ancestor ELSE closure.descendant END WHERE closure.relation = ?4 AND ((closure.descendant = ?1 AND closure.depth > 0) OR (closure.ancestor = ?1 AND closure.depth = 1)) ORDER BY CASE WHEN closure.descendant = ?1 THEN -closure.depth ELSE nodes.id END, nodes.id"#,
            id,
            ancestor,
            child,
            relation
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

impl LeaningRow {
    fn upheld(node: NodeId, statements: &mut Statements<'_>) -> Result<Vec<NodeEdge>, Refusal> {
        let id = node.into_inner();
        let rows = statements.rows(sqlx::query_as!(
            LeaningRow,
            "SELECT node_edges.source, node_edges.relation, node_edges.target, sources.role AS source_role, targets.role AS target_role FROM node_edges JOIN nodes AS sources ON sources.id = node_edges.source JOIN nodes AS targets ON targets.id = node_edges.target WHERE node_edges.source = ?1 OR node_edges.target = ?1 ORDER BY node_edges.source, node_edges.relation, node_edges.target",
            id
        ))?;
        rows.into_iter().map(LeaningRow::edge).collect()
    }

    fn edge(self) -> Result<NodeEdge, Refusal> {
        let relation = NodeRow::decoded(&self.relation).map_err(StoreFailure::from)?;
        let source = NodeRow::decoded(&self.source_role).map_err(StoreFailure::from)?;
        let target = NodeRow::decoded(&self.target_role).map_err(StoreFailure::from)?;
        Ok(NodeEdge::new(NodeId::new(self.source), relation, NodeId::new(self.target)).upheld(source, target)?)
    }
}
