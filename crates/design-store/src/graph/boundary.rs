use design_model::{
    AnyNode, AnyStage, Artifact, Casting, Condition, Dropped, Ending, Entered, Entry, Facet, Kin, Node, NodeEdge,
    NodeId, NodeRelation, Outline, Project, Provenance, Reason, Refining, Role, Stage, Staging, Terminal, Title,
    Transition, Vague,
};
use newtype::Counter;
use port_core::Op;
use store_ports::{Batch, ColumnFailure, Columns, Query, Statements, StoreFailure, Written};

use crate::acyclic::{Acyclic, Closed, Closing, Refines, Refusal, Supersedes};
use crate::{Link, Relative};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolved(NodeId);

#[derive(Clone, Copy, Debug, Op)]
pub struct Resolving(NodeId);

#[derive(Clone, Copy, Debug, Op)]
pub struct ContextNodes;

#[derive(Clone, Copy, Debug, Op)]
pub struct Ended(Resolved);

#[derive(Clone, Debug, Op)]
pub struct Planting {
    root: Root,
    consumed: Option<Resolved>,
}

#[derive(Clone, Debug)]
enum Root {
    Sown(Node<Vague>),
    Found(Resolved),
}

#[derive(Clone, Debug, Op)]
pub struct Restaged {
    node: Resolved,
    transition: Transition,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct Cast {
    node: Resolved,
    casting: Casting,
}

#[derive(Clone, Copy, Debug, Op)]
pub struct Linked {
    source: Resolved,
    relation: NodeRelation,
    target: Resolved,
    provenance: Provenance,
}

#[derive(Clone, Copy, Debug)]
struct EdgeColumns {
    source: NodeId,
    relation: NodeRelation,
    target: NodeId,
    provenance: Provenance,
}

#[derive(Debug, Columns)]
struct IdRow {
    id: NodeId,
}

#[derive(Debug, Columns)]
struct EndingRow {
    ending: Ending,
}

#[derive(Debug, Columns)]
struct NextRow {
    next: NodeId,
}

#[derive(Debug, Columns)]
struct RoleRow {
    role: Role,
}

#[derive(Debug, Columns)]
pub(crate) struct NodeRow {
    id: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    provenance: Provenance,
    title: Option<Title>,
    role: Role,
    role_provenance: Provenance,
    artifact: Option<Artifact>,
    condition: Option<Condition>,
    reason: Option<Reason>,
}

#[derive(Debug, Columns)]
struct OutlineRow {
    id: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    title: Option<Title>,
    parent: Option<NodeId>,
}

#[derive(Debug, Columns)]
pub(crate) struct RelativeRow {
    kin: Kin,
    id: NodeId,
    project: Project,
    facet: Facet,
    stage: Stage,
    provenance: Provenance,
    title: Option<Title>,
    role: Role,
    role_provenance: Provenance,
    artifact: Option<Artifact>,
    condition: Option<Condition>,
    reason: Option<Reason>,
}

#[derive(Debug)]
struct StageColumns {
    stage: Stage,
    artifact: Option<Artifact>,
    condition: Option<Condition>,
    reason: Option<Reason>,
}

#[derive(Debug, Columns)]
pub(crate) struct EdgeRow {
    source: NodeId,
    relation: NodeRelation,
    target: NodeId,
    provenance: Provenance,
}

impl Resolved {
    pub(crate) fn read(id: NodeId, statements: &mut Statements<'_>) -> Result<Resolved, StoreFailure> {
        let row = statements.one(store_ports::query!(
            IdRow,
            r#"
SELECT
  id
FROM
  nodes
WHERE
  id = ?1
"#,
            id: NodeId
        ))?;
        Ok(Resolved(row.id))
    }

    #[must_use]
    pub fn identity(self) -> NodeId {
        self.0
    }
}

impl From<NodeId> for Resolving {
    fn from(id: NodeId) -> Resolving {
        Resolving(id)
    }
}

impl Query for Resolving {
    type Rows = Option<Resolved>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Resolved>, StoreFailure> {
        let id = self.0;
        let rows = statements.rows(store_ports::query!(
            IdRow,
            r#"
SELECT
  id
FROM
  nodes
WHERE
  id = ?1
"#,
            id: NodeId
        ))?;
        Ok(rows.into_iter().next().map(|row| Resolved(row.id)))
    }
}

impl Planting {
    #[must_use]
    pub fn sown(node: Node<Vague>) -> Planting {
        Planting { root: Root::Sown(node), consumed: None }
    }

    #[must_use]
    pub fn found(root: Resolved) -> Planting {
        Planting { root: Root::Found(root), consumed: None }
    }

    #[must_use]
    pub fn consuming(self, consumed: Resolved) -> Planting {
        Planting { consumed: Some(consumed), ..self }
    }
}

impl Root {
    fn sown(&self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        match self {
            Root::Sown(node) => NodeRow::proposed(node, statements),
            Root::Found(_) => Ok(Written::zero()),
        }
    }

    fn resolved(&self, statements: &mut Statements<'_>) -> Result<Resolved, StoreFailure> {
        match self {
            Root::Sown(node) => Resolved::read(node.identity(), statements),
            Root::Found(root) => Ok(*root),
        }
    }
}

impl Restaged {
    #[must_use]
    pub fn new(node: Resolved, transition: Transition) -> Restaged {
        Restaged { node, transition }
    }
}

impl Cast {
    #[must_use]
    pub fn new(node: Resolved, casting: Casting) -> Cast {
        Cast { node, casting }
    }
}

impl Linked {
    #[must_use]
    pub fn new(source: Resolved, relation: NodeRelation, target: Resolved, provenance: Provenance) -> Linked {
        Linked { source, relation, target, provenance }
    }

    fn closed(self, upheld: EdgeColumns, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        match self.relation {
            NodeRelation::Refines => Closing::<Refines>::new(self.source.0, self.target.0)
                .written(statements, |edged| upheld.inserted(edged)),
            NodeRelation::Supersedes => Closing::<Supersedes>::new(self.source.0, self.target.0)
                .written(statements, |edged| Ok::<Written, Refusal>(self.succeeding(upheld, edged)?.inserted(edged)?)),
            NodeRelation::Consumes => Ok(upheld.inserted(statements)?),
        }
    }

    fn succeeding(self, upheld: EdgeColumns, statements: &mut Statements<'_>) -> Result<EdgeColumns, Refusal> {
        let relation = Closed::Refines;
        let old = self.target.0;
        let by = self.source.0;
        let lineage = statements.rows(store_ports::query!(
            IdRow,
            r#"
SELECT
  descendant AS id
FROM
  closure
WHERE
  relation = ?1
  AND ancestor = ?2
  AND descendant = ?3
"#,
            relation: Closed,
            old: NodeId,
            by: NodeId
        ))?;
        lineage.into_iter().next().map_or(Ok(upheld), |_descended| Err(Refusal::Ancestral { old, by }))
    }

    fn upheld(self, statements: &mut Statements<'_>) -> Result<EdgeColumns, Refusal> {
        let source = RoleRow::read(self.source, statements)?;
        let target = RoleRow::read(self.target, statements)?;
        let edge = NodeEdge::new(self.source.0, self.relation, self.target.0).upheld(source, target)?;
        let [source, target] = edge.ends();
        Ok(EdgeColumns { source, relation: edge.relation(), target, provenance: self.provenance })
    }
}

impl RoleRow {
    fn read(end: Resolved, statements: &mut Statements<'_>) -> Result<Role, StoreFailure> {
        let id = end.0;
        let row = statements.one(store_ports::query!(
            RoleRow,
            r#"
SELECT
  role
FROM
  nodes
WHERE
  id = ?1
"#,
            id: NodeId
        ))?;
        Ok(row.role)
    }
}

impl EdgeColumns {
    fn inserted(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let EdgeColumns { source, relation, target, provenance } = self;
        statements.written(store_ports::query!(
            r#"
INSERT
  OR IGNORE INTO node_edges (source, relation, target, provenance)
VALUES
  (?1, ?2, ?3, ?4)
"#,
            source: NodeId,
            relation: NodeRelation,
            target: NodeId,
            provenance: Provenance
        ))
    }
}

impl Batch for Planting {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let sown = self.root.sown(statements)?;
        let root = self.root.resolved(statements)?;
        let linked = self
            .consumed
            .map(|consumed| {
                Linked::new(root, NodeRelation::Consumes, consumed, Provenance::from(Entered::Init)).written(statements)
            })
            .transpose()?;
        Ok(Written::total([sown].into_iter().chain(linked)))
    }
}

impl Batch for Linked {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let upheld = self.upheld(statements)?;
        self.closed(upheld, statements)
    }
}

impl Batch for Restaged {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let id = self.node.0;
        let moved = self.transition.applied(NodeRow::loaded(self.node, statements)?)?;
        let StageColumns { stage, artifact, condition, reason } = StageColumns::from(moved.stage());
        Ok(statements.written(store_ports::query!(
            r#"
UPDATE
  nodes
SET
  stage = ?2,
  artifact = ?3,
  condition = ?4,
  reason = ?5
WHERE
  id = ?1
"#,
            id: NodeId,
            stage: Stage,
            artifact: Option<Artifact>,
            condition: Option<Condition>,
            reason: Option<Reason>
        ))?)
    }
}

impl Batch for Cast {
    type Failure = Refusal;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, Refusal> {
        let id = self.node.0;
        let role = self.casting.role();
        let provenance = self.casting.provenance();
        let written = statements.written(store_ports::query!(
            r#"
UPDATE
  nodes
SET
  role = ?2,
  role_provenance = ?3
WHERE
  id = ?1
"#,
            id: NodeId,
            role: Role,
            provenance: Provenance
        ))?;
        let relinked = EdgeRow::around(id, statements)?
            .into_iter()
            .map(|row| Linked::from(row).written(statements))
            .collect::<Result<Vec<Written>, Refusal>>()?;
        Ok(Written::total([written].into_iter().chain(relinked)))
    }
}

impl From<&AnyStage> for StageColumns {
    fn from(stage: &AnyStage) -> StageColumns {
        let bare = || StageColumns::bare(stage.stage());
        stage.staged(bare()).unwrap_or_else(bare)
    }
}

impl StageColumns {
    fn bare(stage: Stage) -> StageColumns {
        StageColumns { stage, artifact: None, condition: None, reason: None }
    }
}

impl Staging for StageColumns {
    type Output = StageColumns;

    fn terminal(self, terminal: &Terminal) -> StageColumns {
        StageColumns {
            artifact: Some(terminal.artifact().clone()),
            condition: Some(terminal.condition().clone()),
            ..self
        }
    }

    fn dropped(self, dropped: &Dropped) -> StageColumns {
        StageColumns { reason: Some(dropped.reason().clone()), ..self }
    }
}

impl TryFrom<StageColumns> for AnyStage {
    type Error = ColumnFailure;

    fn try_from(columns: StageColumns) -> Result<AnyStage, ColumnFailure> {
        match columns.stage {
            Stage::Vague => Ok(AnyStage::Vague(Vague)),
            Stage::Refining => Ok(AnyStage::Refining(Refining)),
            Stage::Terminal => Ok(AnyStage::Terminal(Terminal::new(
                columns.artifact.ok_or(ColumnFailure::Absent)?,
                columns.condition.ok_or(ColumnFailure::Absent)?,
            ))),
            Stage::Dropped => Ok(AnyStage::Dropped(Dropped::from(columns.reason.ok_or(ColumnFailure::Absent)?))),
        }
    }
}

impl TryFrom<NodeRow> for AnyNode {
    type Error = ColumnFailure;

    fn try_from(row: NodeRow) -> Result<AnyNode, ColumnFailure> {
        let entry = Entry::new(row.project, row.facet, row.provenance);
        let columns =
            StageColumns { stage: row.stage, artifact: row.artifact, condition: row.condition, reason: row.reason };
        let casting = Casting::new(row.role, row.role_provenance);
        let node = AnyNode::loaded(row.id, entry, casting, columns.try_into()?);
        Ok(row.title.into_iter().fold(node, Node::titled))
    }
}

impl From<OutlineRow> for Outline {
    fn from(row: OutlineRow) -> Outline {
        Outline::new(row.id, row.project, row.facet, row.stage).titled(row.title).parented(row.parent)
    }
}

impl TryFrom<RelativeRow> for Relative {
    type Error = ColumnFailure;

    fn try_from(row: RelativeRow) -> Result<Relative, ColumnFailure> {
        let node = NodeRow {
            id: row.id,
            project: row.project,
            facet: row.facet,
            stage: row.stage,
            provenance: row.provenance,
            title: row.title,
            role: row.role,
            role_provenance: row.role_provenance,
            artifact: row.artifact,
            condition: row.condition,
            reason: row.reason,
        };
        Ok(Relative::new(row.kin, AnyNode::try_from(node)?))
    }
}

impl From<EdgeRow> for Linked {
    fn from(row: EdgeRow) -> Linked {
        Linked::new(Resolved(row.source), row.relation, Resolved(row.target), row.provenance)
    }
}

impl From<EdgeRow> for Link {
    fn from(row: EdgeRow) -> Link {
        Link::new(NodeEdge::new(row.source, row.relation, row.target), row.provenance)
    }
}

impl NodeRow {
    pub(crate) fn root(project: &Project, statements: &mut Statements<'_>) -> Result<Option<Resolved>, StoreFailure> {
        let facet = Facet::Project;
        let rows = statements.rows(store_ports::query!(
            IdRow,
            r#"
SELECT
  id
FROM
  nodes
WHERE
  project = ?1
  AND facet = ?2
ORDER BY
  id
LIMIT
  1
"#,
            project: Project,
            facet: Facet
        ))?;
        Ok(rows.into_iter().next().map(|row| Resolved(row.id)))
    }

    pub(crate) fn named(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<AnyNode>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            NodeRow,
            r#"
SELECT
  id,
  project,
  facet,
  stage,
  provenance,
  title,
  role,
  role_provenance,
  artifact,
  condition,
  reason
FROM
  nodes
WHERE
  id = ?1
"#,
            id: NodeId
        ))?;
        Ok(rows.into_iter().map(AnyNode::try_from).collect::<Result<Vec<AnyNode>, ColumnFailure>>()?)
    }

    fn loaded(node: Resolved, statements: &mut Statements<'_>) -> Result<AnyNode, StoreFailure> {
        let id = node.0;
        let row = statements.one(store_ports::query!(
            NodeRow,
            r#"
SELECT
  id,
  project,
  facet,
  stage,
  provenance,
  title,
  role,
  role_provenance,
  artifact,
  condition,
  reason
FROM
  nodes
WHERE
  id = ?1
"#,
            id: NodeId
        ))?;
        Ok(AnyNode::try_from(row)?)
    }

    pub(crate) fn next(statements: &mut Statements<'_>) -> Result<NodeId, StoreFailure> {
        let row = statements.one(store_ports::query!(
            NextRow,
            r#"
SELECT
  COALESCE(MAX(id), 0) + 1 AS "next!: i64"
FROM
  nodes
"#
        ))?;
        Ok(row.next)
    }

    pub(crate) fn proposed(node: &Node<Vague>, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let id = node.identity();
        let entry = node.entry();
        let project = entry.project();
        let facet = entry.facet();
        let stage = Stage::Vague;
        let provenance = entry.provenance();
        let title = node.title().cloned();
        let role = node.casting().role();
        let cast = node.casting().provenance();
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  nodes (
    id,
    project,
    facet,
    stage,
    provenance,
    title,
    role,
    role_provenance
  )
VALUES
  (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#,
            id: NodeId,
            project: Project,
            facet: Facet,
            stage: Stage,
            provenance: Provenance,
            title: Option<Title>,
            role: Role,
            cast: Provenance
        ))
    }
}

impl RelativeRow {
    pub(crate) fn around<R: Acyclic<End = NodeId>>(
        id: NodeId,
        statements: &mut Statements<'_>,
    ) -> Result<Vec<Relative>, StoreFailure> {
        let ancestor = Kin::Ancestor;
        let child = Kin::Child;
        let relation = R::CLOSED;
        let rows = statements.rows(store_ports::query!(
            RelativeRow,
            r#"
SELECT
  CASE
    WHEN closure.descendant = ?1 THEN ?2
    ELSE ?3
  END AS "kin!: String",
  nodes.id,
  nodes.project,
  nodes.facet,
  nodes.stage,
  nodes.provenance,
  nodes.title,
  nodes.role,
  nodes.role_provenance,
  nodes.artifact,
  nodes.condition,
  nodes.reason
FROM
  closure
  JOIN nodes ON nodes.id = CASE
    WHEN closure.descendant = ?1 THEN closure.ancestor
    ELSE closure.descendant
  END
WHERE
  closure.relation = ?4
  AND (
    (
      closure.descendant = ?1
      AND closure.depth > 0
    )
    OR (
      closure.ancestor = ?1
      AND closure.depth = 1
    )
  )
ORDER BY
  CASE
    WHEN closure.descendant = ?1 THEN - closure.depth
    ELSE nodes.id
  END,
  nodes.id
"#,
            id: NodeId,
            ancestor: Kin,
            child: Kin,
            relation: Closed
        ))?;
        Ok(rows.into_iter().map(Relative::try_from).collect::<Result<Vec<Relative>, ColumnFailure>>()?)
    }
}

impl EdgeRow {
    fn around(id: NodeId, statements: &mut Statements<'_>) -> Result<Vec<EdgeRow>, StoreFailure> {
        statements.rows(store_ports::query!(
            EdgeRow,
            r#"
SELECT
  source,
  relation,
  target,
  provenance
FROM
  node_edges
WHERE
  source = ?1
  OR target = ?1
ORDER BY
  source,
  relation,
  target
"#,
            id: NodeId
        ))
    }

    pub(crate) fn links(node: NodeId, statements: &mut Statements<'_>) -> Result<Vec<Link>, StoreFailure> {
        Ok(EdgeRow::around(node, statements)?.into_iter().map(Link::from).collect())
    }
}

impl From<Resolved> for Ended {
    fn from(node: Resolved) -> Ended {
        Ended(node)
    }
}

impl Query for Ended {
    type Rows = Option<Ending>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Ending>, StoreFailure> {
        let node = self.0.0;
        let rows = statements.rows(store_ports::query!(
            EndingRow,
            r#"
SELECT
  ending AS "ending!: String"
FROM
  ended_subtrees
WHERE
  id = ?1
ORDER BY
  ending
LIMIT
  1
"#,
            node: NodeId
        ))?;
        Ok(rows.into_iter().next().map(|row| row.ending))
    }
}

impl Query for ContextNodes {
    type Rows = Vec<Outline>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Outline>, StoreFailure> {
        let relation = Closed::Refines;
        let rows = statements.rows(store_ports::query!(
            OutlineRow,
            r#"
SELECT
  nodes.id,
  nodes.project,
  nodes.facet,
  nodes.stage,
  nodes.title,
  (
    SELECT
      MIN(closure.ancestor)
    FROM
      closure
    WHERE
      closure.relation = ?1
      AND closure.descendant = nodes.id
      AND closure.depth = 1
  ) AS "parent?: i64"
FROM
  nodes
  JOIN live_nodes ON live_nodes.id = nodes.id
ORDER BY
  nodes.project,
  nodes.id
"#,
            relation: Closed
        ))?;
        Ok(rows.into_iter().map(Outline::from).collect())
    }
}
