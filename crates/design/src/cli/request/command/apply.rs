use std::collections::BTreeSet;
use std::path::Path;

use design_model::{
    Entered, Entry, Facet, Node, NodeId, Ordinal, Prefix, Provenance, Segment, SegmentRelation, Sense, SpanRelation,
    Stage, Title,
};
use design_store::{Directed, Directions, NextNode, Prefixed, ShownNode, Spoken};
use file_ports::Files;
use newtype::Newtype;
use store_ports::{Store, Written};
use structurer::{Directive, Directives, SegmentRef, Visit};
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
    next: NodeId,
}

#[derive(Debug)]
struct Grounded {
    directed: Directed,
    heard: Vec<Spoken>,
}

#[derive(Debug)]
struct Resolved {
    directive: Directive,
    grounded: Grounded,
}

#[derive(Debug)]
struct Folded {
    resolved: Vec<Resolved>,
    next: NodeId,
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
        let started = Folded { resolved: Vec::new(), next: store.read(NextNode)? };
        let folded =
            directives.into_iter().try_fold(started, |folded, directive| folded.resolving(store, directive))?;
        let directed = folded.resolved.iter().map(|resolved| resolved.grounded.directed.clone()).collect();
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
        let grounded = directive.visited(&Resolution { store, next: self.next })?;
        let next = grounded.directed.proposed().map_or(self.next, |id| NodeId::new(id.into_inner().saturating_add(1)));
        let resolved = self.resolved.into_iter().chain([Resolved { directive, grounded }]).collect();
        Ok(Folded { resolved, next })
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

    fn node(&self, id: NodeId) -> Result<Node, Failure> {
        let placed = self.store.read(ShownNode::from(id))?;
        Ok(placed.map(|placed| placed.node().clone()).ok_or(Unresolved::Unplaced(id))?)
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

    fn place(self, segment: &SegmentRef, node: NodeId, relation: SpanRelation) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        let node = self.node(node)?.identity();
        Ok(Grounded { directed: Directed::Placed { segment, node, relation }, heard: message })
    }

    fn sense(self, segment: &SegmentRef, sense: Sense) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        Ok(Grounded { directed: Directed::Sensed { segment, sense }, heard: message })
    }

    fn link(self, source: &SegmentRef, target: &SegmentRef, relation: SegmentRelation) -> Result<Grounded, Failure> {
        let source = self.heard(source)?;
        let target = self.heard(target)?;
        let heard = source.message.into_iter().chain(target.message).collect();
        Ok(Grounded { directed: Directed::Related { source: source.segment, target: target.segment, relation }, heard })
    }

    fn propose(self, refines: NodeId, facet: Facet, title: &Title) -> Result<Grounded, Failure> {
        let parent = self.node(refines)?;
        let entry = Entry::new(parent.entry().project().clone(), facet, Stage::Vague, Provenance::from(Entered::Apply));
        let node = Node::new(self.next, entry).titled(title.clone());
        Ok(Grounded { directed: Directed::Proposed { node, parent: parent.identity() }, heard: Vec::new() })
    }

    fn supersede(self, old: NodeId, by: NodeId) -> Result<Grounded, Failure> {
        let old = self.node(old)?.identity();
        let by = self.node(by)?.identity();
        Ok(Grounded { directed: Directed::Superseded { old, by }, heard: Vec::new() })
    }

    fn converse(self, segment: &SegmentRef) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(segment)?;
        Ok(Grounded { directed: Directed::Conversational { segment }, heard: message })
    }
}
