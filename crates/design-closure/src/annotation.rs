mod boundary;

use design_model::{NodeId, Provenance, Segment, SegmentRelation, Sense, SpanRelation, Standing};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Remark {
    Placed { segment: Segment, node: NodeId, relation: SpanRelation },
    Sensed { segment: Segment, sense: Sense },
    Related { source: Segment, target: Segment, relation: SegmentRelation },
    Conversational { segment: Segment },
}

pub trait Remarking {
    type Output;

    fn placed(self, segment: &Segment, node: NodeId, relation: SpanRelation) -> Self::Output;

    fn sensed(self, segment: &Segment, sense: Sense) -> Self::Output;

    fn related(self, source: &Segment, target: &Segment, relation: SegmentRelation) -> Self::Output;

    fn conversational(self, segment: &Segment) -> Self::Output;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation {
    remark: Remark,
    provenance: Provenance,
    standing: Standing,
}

impl Remark {
    pub fn visited<R: Remarking>(&self, remarking: R) -> R::Output {
        match self {
            Remark::Placed { segment, node, relation } => remarking.placed(segment, *node, *relation),
            Remark::Sensed { segment, sense } => remarking.sensed(segment, *sense),
            Remark::Related { source, target, relation } => remarking.related(source, target, *relation),
            Remark::Conversational { segment } => remarking.conversational(segment),
        }
    }

    #[must_use]
    pub fn cited(&self) -> Vec<Segment> {
        match self {
            Remark::Related { source, target, .. } => vec![source.clone(), target.clone()],
            Remark::Placed { segment, .. } | Remark::Sensed { segment, .. } | Remark::Conversational { segment } => {
                vec![segment.clone()]
            }
        }
    }
}

impl Annotation {
    #[must_use]
    pub fn new(remark: Remark, provenance: Provenance, standing: Standing) -> Annotation {
        Annotation { remark, provenance, standing }
    }

    #[must_use]
    pub fn remark(&self) -> &Remark {
        &self.remark
    }

    #[must_use]
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }

    #[must_use]
    pub fn standing(&self) -> Standing {
        self.standing
    }
}
