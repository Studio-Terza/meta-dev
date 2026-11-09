use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

use design_model::{
    AnyNode, Entered, Entry, Facet, Node, NodeId, NodeRelation, Ordinal, Prefix, Provenance, Segment, SegmentRelation,
    Sense, SpanRelation, Straddling, Supersession, Title, Vague,
};
use design_store::{
    AppliedFile, Closes, Covered, Cycle, Directed, Directions, Earlier, JobRepeated, NextNode, Prefixed, Remit, Sees,
    ShownNode, Spoken, Unseen,
};
use dispatch_ports::Reply;
use file_ports::{Bytes, Files, Text};
use newtype::{Counter, Wrapping};
use store_ports::{Store, Written};
use structurer::{Bind, Directive, Directives, Edge, Label, NodeRef, SegmentRef, Visit};
use transcript_ports::{Authorship, Uuid};

use super::Printed;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::output::{Count, Line};
use crate::cli::request::Ground;
use crate::cli::request::command::rendering::Refs;

#[derive(Debug, thiserror::Error)]
pub enum Unresolved {
    #[error("no message in the design store starts as segment {0} does; run design ingest first")]
    Unheard(SegmentRef),
    #[error("several messages start as segment {0} does; write more of its message id")]
    Ambiguous(SegmentRef),
    #[error("the message of segment {0} has no segment at that index")]
    Unsegmented(SegmentRef),
    #[error("segment {0} is not one of this job's segments")]
    Outside(SegmentRef),
    #[error("no node n{} in the design store", .0.into_inner())]
    Unplaced(NodeId),
    #[error("no node labelled {0} is proposed earlier in the file")]
    Unbound(Label),
    #[error("the label {0} already names a node proposed earlier in the file")]
    Rebound(Label),
    #[error("{0}")]
    Straddling(#[from] Straddling),
    #[error(
        "n{} cannot be linked as {relation} n{}, which already {relation} it, directly or through a chain",
        from.into_inner(),
        to.into_inner()
    )]
    Cyclic { from: NodeId, relation: NodeRelation, to: NodeId },
    #[error("n{} cannot supersede n{}, which it refines, directly or through a chain", by.into_inner(), old.into_inner())]
    Ancestral { old: NodeId, by: NodeId },
    #[error("segment {0} cannot be linked to itself")]
    Looped(SegmentRef),
    #[error(
        "{} cannot be linked as {relation} {}, which already {relation} it, directly or through a chain",
        SegmentRef::from(from),
        SegmentRef::from(to)
    )]
    Circular { from: Segment, relation: SegmentRelation, to: Segment },
}

#[derive(Debug)]
pub struct Echo {
    directive: Directive,
    proposed: Option<NodeId>,
}

#[derive(Debug)]
pub enum Application {
    Fresh(Applied),
    Repeated(Earlier),
}

#[derive(Debug)]
pub struct Applied {
    echoes: Vec<Echo>,
    uncovered: Vec<Segment>,
    written: Written,
}

#[derive(Debug)]
struct Fresh<'ground, 'run, F: Files, S: Store>(&'ground Ground<'run, F, S>);

#[derive(Debug)]
pub struct Parsed<'store, S: Store> {
    directives: Directives,
    grounds: Grounds<'store, S>,
}

#[derive(Debug)]
struct Grounds<'store, S: Store> {
    store: &'store S,
    provenance: Provenance,
    remit: Remit,
}

#[derive(Debug)]
struct Resolution<'fold, 'store, S: Store> {
    grounds: &'fold Grounds<'store, S>,
    batch: &'fold Batch,
}

#[derive(Debug)]
struct Batch {
    next: NodeId,
    labels: BTreeMap<Label, NodeId>,
    proposed: BTreeMap<NodeId, AnyNode>,
}

#[derive(Clone, Copy, Debug)]
struct Cycled;

#[derive(Debug)]
struct Labels<'batch>(&'batch BTreeMap<Label, NodeId>);

#[derive(Debug)]
enum Effect {
    Unchanged,
    Proposed { node: Node<Vague>, fresh: Option<Label> },
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

impl Application {
    pub fn read<F: Files, S: Store>(ground: &Ground<'_, F, S>, path: &Path) -> Result<Application, Failure> {
        let contents = ground.read(path).map_err(Failure::Directives)?;
        let store = ground.store();
        let lock = store.lock()?;
        let application = store.read(AppliedFile::from(contents))?.seen(Fresh(ground))?;
        lock.end()?;
        Ok(application)
    }

    pub(crate) fn reported(self, store: &impl Store, out: &mut impl Write) -> Result<Outcome, Failure> {
        let lines = self.lines(store)?;
        Printed::lines(out, &lines).map(|_printed| Outcome::from(self))
    }

    pub(crate) fn lines(&self, store: &impl Store) -> Result<Vec<Line>, Failure> {
        match self {
            Application::Fresh(applied) => applied.lines(store),
            Application::Repeated(earlier) => Ok(vec![Line::from(earlier)]),
        }
    }
}

impl<F: Files, S: Store> Sees for Fresh<'_, '_, F, S> {
    type Output = Result<Application, Failure>;

    fn before(self, earlier: Earlier) -> Result<Application, Failure> {
        Ok(Application::Repeated(earlier))
    }

    fn unseen(self, unseen: Unseen) -> Result<Application, Failure> {
        let text = Text::try_from(unseen.as_ref()).map_err(Failure::Directives)?;
        let directives: Directives = text.as_ref().parse()?;
        Parsed::new(self.0.store(), directives, Provenance::from(Entered::Apply), Remit::Unbounded).unseen(unseen)
    }
}

impl<'store, S: Store> Parsed<'store, S> {
    pub fn new(store: &'store S, directives: Directives, provenance: Provenance, remit: Remit) -> Parsed<'store, S> {
        Parsed { directives, grounds: Grounds { store, provenance, remit } }
    }
}

impl<S: Store> Parsed<'_, S> {
    pub(crate) fn replied(self, reply: &Reply) -> Result<Application, Failure> {
        let store = self.grounds.store;
        store.read(AppliedFile::from(Bytes::from(reply.clone().into_inner().as_str())))?.seen(self)
    }
}

impl<S: Store> Sees for Parsed<'_, S> {
    type Output = Result<Application, Failure>;

    fn before(self, earlier: Earlier) -> Result<Application, Failure> {
        let Grounds { store, remit, .. } = self.grounds;
        store.write(JobRepeated::from(remit))?;
        Ok(Application::Repeated(earlier))
    }

    fn unseen(self, unseen: Unseen) -> Result<Application, Failure> {
        Applied::resolved(self, unseen).map(Application::Fresh)
    }
}

impl From<Application> for Outcome {
    fn from(application: Application) -> Outcome {
        match application {
            Application::Fresh(_applied) => Outcome::Applied,
            Application::Repeated(earlier) => Outcome::AlreadyApplied { earlier },
        }
    }
}

impl Applied {
    fn resolved<S: Store>(parsed: Parsed<'_, S>, unseen: Unseen) -> Result<Applied, Failure> {
        let Parsed { directives, grounds } = parsed;
        let store = grounds.store;
        let started = Folded { resolved: Vec::new(), batch: Batch::starting(store.read(NextNode)?) };
        let folded =
            directives.into_iter().try_fold(started, |folded, directive| folded.resolving(&grounds, directive))?;
        let directed: Vec<Directed> =
            folded.resolved.iter().map(|resolved| resolved.grounded.directed.clone()).collect();
        let written = store.write(Directions::new(grounds.provenance, unseen, directed, grounds.remit))?;
        Applied::covered(store, folded.resolved, written)
    }

    fn covered<S: Store>(store: &S, resolved: Vec<Resolved>, written: Written) -> Result<Applied, Failure> {
        let owned: BTreeSet<Segment> = resolved
            .iter()
            .flat_map(|each| &each.grounded.heard)
            .filter(|spoken| spoken.author() == Authorship::Owner)
            .map(|spoken| spoken.segment().clone())
            .collect();
        let covered =
            store.read(owned.iter().map(|segment| AsRef::<Uuid>::as_ref(segment).clone()).collect::<Covered>())?;
        let uncovered = owned.difference(&covered).cloned().collect();
        Ok(Applied { echoes: resolved.into_iter().map(Echo::from).collect(), uncovered, written })
    }

    fn lines(&self, store: &impl Store) -> Result<Vec<Line>, Failure> {
        let refs = Refs::read(store, self.uncovered.iter().cloned())?;
        Ok(self
            .echoes
            .iter()
            .map(Line::from)
            .chain(refs.uncovered(&self.uncovered))
            .chain([Line::Written(Count::new(self.written.count()))])
            .collect())
    }
}

impl From<&Echo> for Line {
    fn from(echo: &Echo) -> Line {
        let directive = echo.directive.clone();
        echo.proposed.map_or_else(
            || Line::Applied(directive.clone()),
            |node| Line::Proposed { node, directive: directive.clone() },
        )
    }
}

impl From<Resolved> for Echo {
    fn from(resolved: Resolved) -> Echo {
        Echo { proposed: resolved.grounded.directed.proposed(), directive: resolved.directive }
    }
}

impl Folded {
    fn resolving<S: Store>(self, grounds: &Grounds<'_, S>, directive: Directive) -> Result<Folded, Failure> {
        let grounded = directive.visited(&Resolution { grounds, batch: &self.batch })?;
        let batch = self.batch.after(&grounded.effect);
        let resolved = self.resolved.into_iter().chain([Resolved { directive, grounded }]).collect();
        Ok(Folded { resolved, batch })
    }
}

impl Batch {
    fn starting(next: NodeId) -> Batch {
        Batch { next, labels: BTreeMap::new(), proposed: BTreeMap::new() }
    }

    fn after(self, effect: &Effect) -> Batch {
        match effect {
            Effect::Proposed { node, fresh } => Batch {
                next: NodeId::new(node.identity().into_inner().saturating_add(1)),
                labels: self.labels.into_iter().chain(fresh.clone().map(|label| (label, node.identity()))).collect(),
                proposed: self.proposed.into_iter().chain([(node.identity(), node.clone().any())]).collect(),
            },
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

impl<S: Store> Resolution<'_, '_, S> {
    fn heard(&self, reference: &SegmentRef, remit: &Remit) -> Result<Heard, Failure> {
        let prefix: &Prefix = reference.as_ref();
        let spoken = self.grounds.store.read(Prefixed::from(prefix.clone()))?;
        let messages: BTreeSet<&Uuid> = spoken.iter().map(|heard| heard.segment().as_ref()).collect();
        let message = Heard::only(reference, messages)?;
        let wanted = Segment::new(message, Ordinal::from(reference));
        let segment = spoken
            .iter()
            .map(Spoken::segment)
            .find(|segment| **segment == wanted)
            .cloned()
            .ok_or_else(|| Unresolved::Unsegmented(reference.clone()))?;
        let admitted = remit.admitted(segment).ok_or_else(|| Unresolved::Outside(reference.clone()))?;
        Ok(Heard { segment: admitted, message: spoken })
    }

    fn node(&self, reference: &NodeRef) -> Result<AnyNode, Failure> {
        let id = reference.bound(Labels(&self.batch.labels))?;
        self.batch.proposed.get(&id).cloned().map_or_else(|| self.stored(id), Ok)
    }

    fn stored(&self, id: NodeId) -> Result<AnyNode, Failure> {
        let placed = self.grounds.store.read(ShownNode::from(id))?;
        Ok(placed.map(|shown| shown.node().clone()).ok_or(Unresolved::Unplaced(id))?)
    }
}

impl From<Cycle> for Unresolved {
    fn from(cycle: Cycle) -> Unresolved {
        cycle.closed(Cycled)
    }
}

impl Closes for Cycled {
    type Output = Unresolved;

    fn nodes(self, from: NodeId, relation: NodeRelation, to: NodeId) -> Unresolved {
        Unresolved::Cyclic { from, relation, to }
    }

    fn segments(self, from: Segment, relation: SegmentRelation, to: Segment) -> Unresolved {
        Unresolved::Circular { from, relation, to }
    }
}

impl Heard {
    fn only(reference: &SegmentRef, messages: BTreeSet<&Uuid>) -> Result<Uuid, Unresolved> {
        let found: Vec<Uuid> = messages.into_iter().cloned().collect();
        <[Uuid; 1]>::try_from(found).map(|[message]| message).map_err(|several_or_none| {
            several_or_none.first().map_or_else(
                || Unresolved::Unheard(reference.clone()),
                |_several| Unresolved::Ambiguous(reference.clone()),
            )
        })
    }
}

impl<S: Store> Visit for &Resolution<'_, '_, S> {
    type Output = Result<Grounded, Failure>;

    fn place(self, cited: &SegmentRef, named: &NodeRef, relation: SpanRelation) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(cited, &self.grounds.remit)?;
        let node = self.node(named)?.identity();
        Ok(Grounded {
            directed: Directed::Placed { segment, node, relation },
            heard: message,
            effect: Effect::Unchanged,
        })
    }

    fn sense(self, cited: &SegmentRef, sense: Sense) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(cited, &self.grounds.remit)?;
        Ok(Grounded { directed: Directed::Sensed { segment, sense }, heard: message, effect: Effect::Unchanged })
    }

    fn link(self, from: &SegmentRef, to: &SegmentRef, relation: SegmentRelation) -> Result<Grounded, Failure> {
        let looped = Unresolved::Looped(from.clone());
        let source = self.heard(from, &self.grounds.remit)?;
        let target = self.heard(to, &Remit::Unbounded)?;
        let heard = source.message.into_iter().chain(target.message).collect();
        let edge = Edge::new(source.segment, target.segment).map_err(|_looped| looped)?;
        let directed =
            edge.ends(|start, end| Directed::Related { source: start.clone(), target: end.clone(), relation });
        Ok(Grounded { directed, heard, effect: Effect::Unchanged })
    }

    fn propose(
        self,
        labelled: Option<&Label>,
        refines: &NodeRef,
        facet: Facet,
        title: &Title,
    ) -> Result<Grounded, Failure> {
        let fresh = labelled.map(|label| self.batch.unbound(label)).transpose()?;
        let parent = self.node(refines)?;
        let entry = Entry::new(parent.entry().project().clone(), facet, self.grounds.provenance);
        let node = Node::proposed(self.batch.next, entry, parent.casting()).titled(title.clone());
        let effect = Effect::Proposed { node: node.clone(), fresh };
        Ok(Grounded { directed: Directed::Proposed { node, parent: parent.identity() }, heard: Vec::new(), effect })
    }

    fn supersede(self, superseded: &NodeRef, superseding: &NodeRef) -> Result<Grounded, Failure> {
        let old = self.node(superseded)?;
        let by = self.node(superseding)?;
        let supersession = Supersession::within(&old, &by).map_err(Unresolved::from)?;
        Ok(Grounded { directed: Directed::Superseded(supersession), heard: Vec::new(), effect: Effect::Unchanged })
    }

    fn converse(self, cited: &SegmentRef) -> Result<Grounded, Failure> {
        let Heard { segment, message } = self.heard(cited, &self.grounds.remit)?;
        Ok(Grounded { directed: Directed::Conversational { segment }, heard: message, effect: Effect::Unchanged })
    }
}
