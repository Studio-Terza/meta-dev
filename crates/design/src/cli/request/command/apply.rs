use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use design_model::{
    Entered, Entry, Facet, Node, NodeId, Ordinal, Prefix, Provenance, Segment, SegmentRelation, Sense, SpanRelation,
    Stage, Straddling, Supersession, Title,
};
use design_store::{Directed, Directions, NextNode, Prefixed, ShownNode, Spoken, SupersededBy};
use file_ports::Files;
use newtype::Newtype;
use store_ports::{Store, Written};
use structurer::{Bind, Directive, Directives, Label, NodeRef, SegmentRef, Visit};
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
    supersessions: Vec<Supersession>,
}

#[derive(Debug)]
struct Chain<'store, S: Store> {
    store: &'store S,
    pending: &'store [Supersession],
}

#[derive(Debug)]
struct Labels<'batch>(&'batch BTreeMap<Label, NodeId>);

#[derive(Debug)]
enum Effect {
    Unchanged,
    Proposed { node: Node, fresh: Option<Label> },
    Superseded(Supersession),
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
        lock.end()?;
        Ok(Applied::covered(folded.resolved, written))
    }

    fn covered(resolved: Vec<Resolved>, written: Written) -> Applied {
        let mentioned: BTreeSet<&Segment> =
            resolved.iter().flat_map(|resolved| resolved.grounded.directed.segments()).collect();
        let owned: BTreeSet<&Segment> = resolved
            .iter()
            .flat_map(|resolved| &resolved.grounded.heard)
            .filter(|spoken| spoken.author() == Authorship::Owner)
            .map(Spoken::segment)
            .collect();
        let uncovered = owned.difference(&mentioned).map(|segment| (*segment).clone()).collect();
        Applied { echoes: resolved.into_iter().map(Echo::from).collect(), uncovered, written }
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
        Batch { next, labels: BTreeMap::new(), proposed: BTreeMap::new(), supersessions: Vec::new() }
    }

    fn after(self, effect: &Effect) -> Batch {
        match effect {
            Effect::Proposed { node, fresh } => Batch {
                next: NodeId::new(node.identity().into_inner().saturating_add(1)),
                labels: self.labels.into_iter().chain(fresh.clone().map(|label| (label, node.identity()))).collect(),
                proposed: self.proposed.into_iter().chain([(node.identity(), node.clone())]).collect(),
                ..self
            },
            Effect::Superseded(supersession) => {
                Batch { supersessions: self.supersessions.into_iter().chain([*supersession]).collect(), ..self }
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
}

impl<S: Store> Chain<'_, S> {
    fn acyclic(&self, supersession: Supersession) -> Result<Supersession, Failure> {
        let [old, by] = supersession.ends();
        let reached = self.reached(old)?;
        Ok(reached.get(&by).map_or(Ok(supersession), |_cycle| Err(Unresolved::Cyclic { old, by }))?)
    }

    fn reached(&self, from: NodeId) -> Result<BTreeSet<NodeId>, Failure> {
        let stored = self.store.read(SupersededBy::from(from))?;
        let onward = self
            .pending
            .iter()
            .map(|pending| pending.ends())
            .filter(|[_old, by]| stored.contains(by))
            .map(|[old, _by]| self.reached(old))
            .collect::<Result<Vec<BTreeSet<NodeId>>, Failure>>()?;
        Ok(stored.into_iter().chain(onward.into_iter().flatten()).collect())
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
        let source = self.heard(source)?;
        let target = self.heard(target)?;
        let heard = source.message.into_iter().chain(target.message).collect();
        let directed = Directed::Related { source: source.segment, target: target.segment, relation };
        Ok(Grounded { directed, heard, effect: Effect::Unchanged })
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
        let rooted = Supersession::within(&self.node(old)?, &self.node(by)?).map_err(Unresolved::from)?;
        let supersession = Chain { store: self.store, pending: &self.batch.supersessions }.acyclic(rooted)?;
        let effect = Effect::Superseded(supersession);
        Ok(Grounded { directed: Directed::Superseded(supersession), heard: Vec::new(), effect })
    }

    fn converse(self, segment: &SegmentRef) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        Ok(Grounded { directed: Directed::Conversational { segment }, heard: message, effect: Effect::Unchanged })
    }
}
