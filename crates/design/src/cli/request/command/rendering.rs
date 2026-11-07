use design_model::{
    Dropped, Entered, NodeId, Ordinal, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Staging, Terminal,
};
use design_store::{
    Annotation, Earlier, Hit, Placed, Placement, Prefixes, RelationName, Remarking, ShapeName, Shortest, Stored,
};
use newtype::Newtype;
use store_ports::{Store, StoreFailure};
use stream::{Anchors, SegmentId};
use structurer::SegmentRef;
use transcript_ports::{Authorship, Body, Instant, SessionId, Uuid};

use crate::cli::output::{Anchored, Line, Remark, Summary};

#[derive(Debug)]
pub struct Refs(Shortest);

#[derive(Clone, Copy, Debug)]
struct Details;

#[derive(Clone, Copy, Debug)]
struct Remarks<'refs>(&'refs Refs);

#[derive(Clone, Copy, Debug)]
struct Ends<'refs>(&'refs Refs);

impl Refs {
    pub fn read(store: &impl Store, segments: impl IntoIterator<Item = Segment>) -> Result<Refs, StoreFailure> {
        let messages: Prefixes = segments.into_iter().map(|segment| AsRef::<Uuid>::as_ref(&segment).clone()).collect();
        Ok(Refs(store.read(messages)?))
    }

    pub fn placed(store: &impl Store, placed: &Placed) -> Result<Vec<Line>, StoreFailure> {
        let refs = Refs::read(store, placed.placements().iter().map(Placement::segment))?;
        Ok(refs.neighbourhood(placed))
    }

    pub fn stored(store: &impl Store, stored: &Stored) -> Result<Vec<Line>, StoreFailure> {
        let uuid: &Uuid = stored.message().as_ref();
        let segments = stored.segments().iter().map(|segment| Segment::new(uuid.clone(), segment.ordinal()));
        let remarked = stored.annotations().iter().flat_map(|annotation| annotation.remark().cited());
        let linked = stored.links().iter().flat_map(|link| link.anchors().map(|anchor| anchor.message().clone()));
        let messages: Prefixes =
            segments.chain(remarked).map(|segment| AsRef::<Uuid>::as_ref(&segment).clone()).chain(linked).collect();
        Ok(Refs(store.read(messages)?).message(stored))
    }

    fn reference(&self, segment: &Segment) -> SegmentRef {
        SegmentRef::new(self.0.prefix(segment.as_ref()), Ordinal::from(segment))
    }

    fn neighbourhood(&self, placed: &Placed) -> Vec<Line> {
        let node = placed.node();
        let casting = node.casting();
        [Line::Node(Summary::from(node))]
            .into_iter()
            .chain(node.stage().staged(Details).unwrap_or_default())
            .chain(
                placed
                    .relatives()
                    .iter()
                    .map(|relative| Line::Relative { kin: relative.kin(), node: Summary::from(relative.node()) }),
            )
            .chain(
                placed
                    .links()
                    .iter()
                    .map(|link| Line::Edge { edge: link.edge(), entered: Entered::from(link.provenance()) }),
            )
            .chain([Line::Role { role: casting.role(), entered: Entered::from(casting.provenance()) }])
            .chain(placed.placements().iter().map(|placement| {
                let author: &Authorship = placement.as_ref();
                let provenance: &Provenance = placement.as_ref();
                Line::Placed {
                    relation: placement.relation(),
                    segment: self.reference(&placement.segment()),
                    author: *author,
                    sense: placement.sense(),
                    entered: Entered::from(*provenance),
                    body: placement.body().clone(),
                }
            }))
            .collect()
    }

    fn message(&self, stored: &Stored) -> Vec<Line> {
        let message = stored.message();
        let uuid: &Uuid = message.as_ref();
        let author: &Authorship = message.as_ref();
        let session: &SessionId = message.as_ref();
        let stamp: &Instant = message.as_ref();
        [Line::Message { uuid: uuid.clone(), author: *author, session: session.clone(), stamp: stamp.clone() }]
            .into_iter()
            .chain(stored.segments().iter().map(|segment| {
                let shape: &ShapeName = segment.as_ref();
                Line::Segment {
                    segment: self.reference(&Segment::new(uuid.clone(), segment.ordinal())),
                    shape: shape.clone(),
                    body: Body::from(segment),
                }
            }))
            .chain(stored.links().iter().map(|link| {
                let [source, target] = link.anchors();
                let relation: &RelationName = link.as_ref();
                Line::Link {
                    source: source.visited(Ends(self)),
                    relation: relation.clone(),
                    target: target.visited(Ends(self)),
                }
            }))
            .chain(stored.annotations().iter().map(|annotation: &Annotation| Line::Annotated {
                remark: annotation.remark().visited(Remarks(self)),
                standing: annotation.standing(),
                entered: Entered::from(annotation.provenance()),
            }))
            .collect()
    }

    pub fn hits(&self, hits: &[Hit]) -> Vec<Line> {
        hits.iter()
            .map(|hit| {
                let author: &Authorship = hit.as_ref();
                let stamp: &Instant = hit.as_ref();
                Line::Hit {
                    segment: self.reference(&Segment::from(hit)),
                    author: *author,
                    stamp: stamp.clone(),
                    body: Body::from(hit),
                }
            })
            .collect()
    }

    pub fn uncovered(&self, segments: &[Segment]) -> Vec<Line> {
        segments.iter().map(|segment| Line::Uncovered(self.reference(segment))).collect()
    }
}

impl From<&Earlier> for Line {
    fn from(earlier: &Earlier) -> Line {
        Line::Repeated { file: earlier.identity(), stamp: earlier.stamp().clone() }
    }
}

impl Staging for Details {
    type Output = Vec<Line>;

    fn terminal(self, terminal: &Terminal) -> Vec<Line> {
        vec![Line::Artifact(terminal.artifact().clone()), Line::Condition(terminal.condition().clone())]
    }

    fn dropped(self, dropped: &Dropped) -> Vec<Line> {
        vec![Line::Reason(dropped.reason().clone())]
    }
}

impl Remarking for Remarks<'_> {
    type Output = Remark;

    fn placed(self, segment: &Segment, node: NodeId, relation: SpanRelation) -> Remark {
        Remark::Place { segment: self.0.reference(segment), relation, node }
    }

    fn sensed(self, segment: &Segment, sense: Sense) -> Remark {
        Remark::Sense { segment: self.0.reference(segment), sense }
    }

    fn related(self, source: &Segment, target: &Segment, relation: SegmentRelation) -> Remark {
        Remark::Link { source: self.0.reference(source), relation, target: self.0.reference(target) }
    }

    fn conversational(self, segment: &Segment) -> Remark {
        Remark::Conversational { segment: self.0.reference(segment) }
    }
}

impl Anchors for Ends<'_> {
    type Output = Anchored;

    fn message(self, message: &Uuid) -> Anchored {
        Anchored::Message(self.0.0.prefix(message))
    }

    fn segment(self, segment: &SegmentId) -> Anchored {
        Anchored::Segment(
            self.0.reference(&Segment::new(segment.message().clone(), Ordinal::new(segment.ordinal().into_inner()))),
        )
    }
}
