use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use design_model::{
    Entered, Entry, Facet, Node, NodeId, Ordinal, Prefix, Provenance, Segment, SegmentRelation, Sense, SpanRelation,
    Stage, Straddling, Supersession, Title,
};
use design_store::{Covered, Directed, Directions, NextNode, Ordered, Prefixed, ShownNode, Spoken, SupersededBy};
use file_ports::Files;
use newtype::Newtype;
use store_ports::{Store, Written};
use structurer::{Bind, Directive, Directives, Edge, Label, NodeRef, SegmentRef, Visit};
use transcript_ports::{Authorship, Uuid};

use crate::cli::failure::Failure;
use crate::cli::rendering::Id;
use crate::cli::request::Ground;

#[derive(Debug, thiserror::Error)]
pub enum Unresolved {
    #[error("no message in the design store starts as segment {0} does; run design ingest first")]
    Unheard(SegmentRef),
    #[error("several messages start as segment {0} does; write more of its message id")]
    Ambiguous(SegmentRef),
    #[error("the message of segment {0} has no segment at that index")]
    Unsegmented(SegmentRef),
    #[error("no node {} in the design store", Id::from(*.0))]
    Unplaced(NodeId),
    #[error("no node labelled {0} is proposed earlier in the file")]
    Unbound(Label),
    #[error("the label {0} already names a node proposed earlier in the file")]
    Rebound(Label),
    #[error("{0}")]
    Straddling(#[from] Straddling),
    #[error(
        "{} cannot be superseded by {}, which {} already supersedes, directly or through a chain",
        Id::from(*old),
        Id::from(*by),
        Id::from(*old)
    )]
    Cyclic { old: NodeId, by: NodeId },
    #[error("segment {0} cannot be linked to itself")]
    Looped(SegmentRef),
    #[error(
        "{later} cannot be linked as {relation} {earlier}, which already {relation} it, directly or through a chain"
    )]
    Circular { later: SegmentRef, relation: SegmentRelation, earlier: SegmentRef },
}

#[derive(Debug)]
pub struct Echo {
    directive: Directive,
    proposed: Option<NodeId>,
}

#[derive(Debug)]
pub struct Applied {
    echoes: Vec<Echo>,
    uncovered: Vec<Segment>,
    written: Written,
}

#[derive(Debug)]
struct Resolution<'store, S: Store> {
    store: &'store S,
    batch: &'store Batch,
}

#[derive(Debug)]
struct Batch {
    next: NodeId,
    labels: BTreeMap<Label, NodeId>,
    proposed: BTreeMap<NodeId, Node>,
    superseding: Vec<Edge<NodeId>>,
    links: Vec<Linking>,
}

#[derive(Clone, Debug)]
struct Linking {
    relation: SegmentRelation,
    edge: Edge<Segment>,
}

#[derive(Debug)]
struct Chain<'store, S: Store, O: Ordering<S>> {
    store: &'store S,
    order: O,
    pending: Vec<Edge<O::End>>,
}

#[derive(Clone, Copy, Debug)]
struct Superseding;

#[derive(Clone, Copy, Debug)]
struct Sequence(SegmentRelation);

trait Ordering<S: Store> {
    type End: Ord + Clone;

    fn reached(&self, store: &S, from: Self::End) -> Result<BTreeSet<Self::End>, Failure>;

    fn pending(&self, batch: &Batch) -> Vec<Edge<Self::End>>;

    fn cyclic(&self, source: Self::End, target: Self::End) -> Unresolved;
}

#[derive(Debug)]
struct Labels<'batch>(&'batch BTreeMap<Label, NodeId>);

#[derive(Debug)]
enum Effect {
    Unchanged,
    Proposed { node: Node, fresh: Option<Label> },
    Superseded(Edge<NodeId>),
    Linked(Linking),
}

#[derive(Debug)]
struct Grounded {
    directed: Directed,
    heard: Vec<Spoken>,
    effect: Effect,
}

#[derive(Debug)]
struct Resolved {
    directive: Directive,
    grounded: Grounded,
}

#[derive(Debug)]
struct Folded {
    resolved: Vec<Resolved>,
    batch: Batch,
}

#[derive(Debug)]
struct Heard {
    segment: Segment,
    message: Vec<Spoken>,
}

impl Applied {
    pub fn read<F: Files, S: Store>(ground: &Ground<'_, F, S>, path: &Path) -> Result<Applied, Failure> {
        let directives: Directives = ground.read(path)?.as_ref().parse()?;
        let store = ground.store();
        let lock = store.lock()?;
        let started = Folded { resolved: Vec::new(), batch: Batch::starting(store.read(NextNode)?) };
        let folded =
            directives.into_iter().try_fold(started, |folded, directive| folded.resolving(store, directive))?;
        let directed: Vec<Directed> =
            folded.resolved.iter().map(|resolved| resolved.grounded.directed.clone()).collect();
        let written = store.write(Directions::new(Provenance::from(Entered::Apply), directed))?;
        let applied = Applied::covered(store, folded.resolved, written)?;
        lock.end()?;
        Ok(applied)
    }

    fn covered<S: Store>(store: &S, resolved: Vec<Resolved>, written: Written) -> Result<Applied, Failure> {
        let owned: BTreeSet<Segment> = resolved
            .iter()
            .flat_map(|resolved| &resolved.grounded.heard)
            .filter(|spoken| spoken.author() == Authorship::Owner)
            .map(|spoken| spoken.segment().clone())
            .collect();
        let covered =
            store.read(owned.iter().map(|segment| AsRef::<Uuid>::as_ref(segment).clone()).collect::<Covered>())?;
        let uncovered = owned.difference(&covered).cloned().collect();
        Ok(Applied { echoes: resolved.into_iter().map(Echo::from).collect(), uncovered, written })
    }

    pub(crate) fn echoes(&self) -> &[Echo] {
        &self.echoes
    }

    pub(crate) fn uncovered(&self) -> &[Segment] {
        &self.uncovered
    }

    pub(crate) fn written(&self) -> Written {
        self.written
    }
}

impl Echo {
    pub(crate) fn directive(&self) -> &Directive {
        &self.directive
    }

    pub(crate) fn proposed(&self) -> Option<NodeId> {
        self.proposed
    }
}

impl From<Resolved> for Echo {
    fn from(resolved: Resolved) -> Echo {
        Echo { proposed: resolved.grounded.directed.proposed(), directive: resolved.directive }
    }
}

impl Folded {
    fn resolving<S: Store>(self, store: &S, directive: Directive) -> Result<Folded, Failure> {
        let grounded = directive.visited(&Resolution { store, batch: &self.batch })?;
        let batch = self.batch.after(&grounded.effect);
        let resolved = self.resolved.into_iter().chain([Resolved { directive, grounded }]).collect();
        Ok(Folded { resolved, batch })
    }
}

impl Batch {
    fn starting(next: NodeId) -> Batch {
        Batch { next, labels: BTreeMap::new(), proposed: BTreeMap::new(), superseding: Vec::new(), links: Vec::new() }
    }

    fn after(self, effect: &Effect) -> Batch {
        match effect {
            Effect::Proposed { node, fresh } => Batch {
                next: NodeId::new(node.identity().into_inner().saturating_add(1)),
                labels: self.labels.into_iter().chain(fresh.clone().map(|label| (label, node.identity()))).collect(),
                proposed: self.proposed.into_iter().chain([(node.identity(), node.clone())]).collect(),
                ..self
            },
            Effect::Superseded(edge) => {
                Batch { superseding: self.superseding.into_iter().chain([edge.clone()]).collect(), ..self }
            }
            Effect::Linked(linking) => {
                Batch { links: self.links.into_iter().chain([linking.clone()]).collect(), ..self }
            }
            Effect::Unchanged => self,
        }
    }

    fn unbound(&self, label: &Label) -> Result<Label, Unresolved> {
        self.labels.get(label).map_or_else(|| Ok(label.clone()), |_bound| Err(Unresolved::Rebound(label.clone())))
    }
}

impl Bind for Labels<'_> {
    type Output = Result<NodeId, Unresolved>;

    fn stored(self, id: NodeId) -> Result<NodeId, Unresolved> {
        Ok(id)
    }

    fn labelled(self, label: &Label) -> Result<NodeId, Unresolved> {
        self.0.get(label).copied().ok_or_else(|| Unresolved::Unbound(label.clone()))
    }
}

impl<S: Store> Resolution<'_, S> {
    fn heard(&self, reference: &SegmentRef) -> Result<Heard, Failure> {
        let prefix: &Prefix = reference.as_ref();
        let spoken = self.store.read(Prefixed::from(prefix.clone()))?;
        let messages: BTreeSet<&Uuid> = spoken.iter().map(|spoken| spoken.segment().as_ref()).collect();
        let message = Heard::only(reference, messages)?;
        let wanted = Segment::new(message, Ordinal::from(reference));
        let segment = spoken
            .iter()
            .map(Spoken::segment)
            .find(|segment| **segment == wanted)
            .cloned()
            .ok_or_else(|| Unresolved::Unsegmented(reference.clone()))?;
        Ok(Heard { segment, message: spoken })
    }

    fn node(&self, reference: &NodeRef) -> Result<Node, Failure> {
        let id = reference.bound(Labels(&self.batch.labels))?;
        self.batch.proposed.get(&id).cloned().map_or_else(|| self.stored(id), Ok)
    }

    fn stored(&self, id: NodeId) -> Result<Node, Failure> {
        let placed = self.store.read(ShownNode::from(id))?;
        Ok(placed.map(|placed| placed.node().clone()).ok_or(Unresolved::Unplaced(id))?)
    }

    fn acyclic<O: Ordering<S>>(&self, order: O, edge: Edge<O::End>) -> Result<Edge<O::End>, Failure> {
        let pending = order.pending(self.batch);
        Chain { store: self.store, order, pending }.acyclic(edge)
    }
}

impl<S: Store, O: Ordering<S>> Chain<'_, S, O> {
    fn acyclic(&self, edge: Edge<O::End>) -> Result<Edge<O::End>, Failure> {
        let reached = edge.ends(|_source, target| self.reached(target.clone()))?;
        let cycle = edge.ends(|source, target| {
            reached.get(source).map(|_closed| self.order.cyclic(source.clone(), target.clone()))
        });
        Ok(cycle.map_or(Ok(edge), Err)?)
    }

    fn reached(&self, from: O::End) -> Result<BTreeSet<O::End>, Failure> {
        let stored = self.order.reached(self.store, from)?;
        let onward = self
            .pending
            .iter()
            .filter(|pending| pending.ends(|source, _target| stored.contains(source)))
            .map(|pending| pending.ends(|_source, target| self.reached(target.clone())))
            .collect::<Result<Vec<BTreeSet<O::End>>, Failure>>()?;
        Ok(stored.into_iter().chain(onward.into_iter().flatten()).collect())
    }
}

impl<S: Store> Ordering<S> for Superseding {
    type End = NodeId;

    fn reached(&self, store: &S, from: NodeId) -> Result<BTreeSet<NodeId>, Failure> {
        Ok(store.read(SupersededBy::from(from))?)
    }

    fn pending(&self, batch: &Batch) -> Vec<Edge<NodeId>> {
        batch.superseding.clone()
    }

    fn cyclic(&self, by: NodeId, old: NodeId) -> Unresolved {
        Unresolved::Cyclic { old, by }
    }
}

impl Sequence {
    fn sequenced<S: Store>(
        resolution: &Resolution<'_, S>,
        relation: SegmentRelation,
        joined: Edge<Segment>,
    ) -> Result<Edge<Segment>, Failure> {
        match relation {
            SegmentRelation::Supersedes | SegmentRelation::Corrects => resolution.acyclic(Sequence(relation), joined),
            SegmentRelation::Answers | SegmentRelation::RefersTo => Ok(joined),
        }
    }
}

impl<S: Store> Ordering<S> for Sequence {
    type End = Segment;

    fn reached(&self, store: &S, from: Segment) -> Result<BTreeSet<Segment>, Failure> {
        Ok(store.read(Ordered::along(self.0, from))?)
    }

    fn pending(&self, batch: &Batch) -> Vec<Edge<Segment>> {
        batch.links.iter().filter(|linking| linking.relation == self.0).map(|linking| linking.edge.clone()).collect()
    }

    fn cyclic(&self, source: Segment, target: Segment) -> Unresolved {
        Unresolved::Circular { later: SegmentRef::from(&source), relation: self.0, earlier: SegmentRef::from(&target) }
    }
}

impl Heard {
    fn only(reference: &SegmentRef, messages: BTreeSet<&Uuid>) -> Result<Uuid, Unresolved> {
        let found: Vec<Uuid> = messages.into_iter().cloned().collect();
        <[Uuid; 1]>::try_from(found).map(|[message]| message).map_err(|found| {
            found.first().map_or_else(
                || Unresolved::Unheard(reference.clone()),
                |_several| Unresolved::Ambiguous(reference.clone()),
            )
        })
    }
}

impl<S: Store> Visit for &Resolution<'_, S> {
    type Output = Result<Grounded, Failure>;

    fn place(self, segment: &SegmentRef, node: &NodeRef, relation: SpanRelation) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        let node = self.node(node)?.identity();
        Ok(Grounded {
            directed: Directed::Placed { segment, node, relation },
            heard: message,
            effect: Effect::Unchanged,
        })
    }

    fn sense(self, segment: &SegmentRef, sense: Sense) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        Ok(Grounded { directed: Directed::Sensed { segment, sense }, heard: message, effect: Effect::Unchanged })
    }

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> Result<Grounded, Failure> {
        let looped = Unresolved::Looped(source.clone());
        let source = self.heard(source)?;
        let target = self.heard(target)?;
        let heard = source.message.into_iter().chain(target.message).collect();
        let joined = Edge::new(source.segment, target.segment).map_err(|_looped| looped)?;
        let edge = Sequence::sequenced(self, relation, joined)?;
        let directed =
            edge.ends(|source, target| Directed::Related { source: source.clone(), target: target.clone(), relation });
        Ok(Grounded { directed, heard, effect: Effect::Linked(Linking { relation, edge }) })
    }

    fn propose(
        self,
        fresh: Option<&Label>,
        refines: &NodeRef,
        facet: Facet,
        title: &Title,
    ) -> Result<Grounded, Failure> {
        let fresh = fresh.map(|label| self.batch.unbound(label)).transpose()?;
        let parent = self.node(refines)?;
        let entry = Entry::new(parent.entry().project().clone(), facet, Stage::Vague, Provenance::from(Entered::Apply));
        let node = Node::new(self.batch.next, entry).titled(title.clone());
        let effect = Effect::Proposed { node: node.clone(), fresh };
        Ok(Grounded { directed: Directed::Proposed { node, parent: parent.identity() }, heard: Vec::new(), effect })
    }

    fn supersede(self, old: &NodeRef, by: &NodeRef) -> Result<Grounded, Failure> {
        let old = self.node(old)?;
        let by = self.node(by)?;
        let cycle = Unresolved::Cyclic { old: old.identity(), by: by.identity() };
        let joined = Edge::new(by.identity(), old.identity()).map_err(|_looped| cycle)?;
        let supersession = Supersession::within(&old, &by).map_err(Unresolved::from)?;
        let effect = Effect::Superseded(self.acyclic(Superseding, joined)?);
        Ok(Grounded { directed: Directed::Superseded(supersession), heard: Vec::new(), effect })
    }

    fn converse(self, segment: &SegmentRef) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        Ok(Grounded { directed: Directed::Conversational { segment }, heard: message, effect: Effect::Unchanged })
    }
}
