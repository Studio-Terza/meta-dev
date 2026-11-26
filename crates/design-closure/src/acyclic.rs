mod boundary;

use design_model::{NodeId, NodeRelation, Segment, SegmentRelation};
use newtype::Listed;
use store_ports::{Execute, StoreFailure};
use strum::{Display, IntoStaticStr, VariantArray};

pub use boundary::{Closing, Keyed};

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case")]
pub enum Closed {
    Refines,
    Supersedes,
    SegmentSupersedes,
    SegmentCorrects,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cycle {
    Nodes { source: NodeId, relation: NodeRelation, target: NodeId },
    Segments { source: Segment, relation: SegmentRelation, target: Segment },
}

pub trait Closes {
    type Output;

    fn nodes(self, source: NodeId, relation: NodeRelation, target: NodeId) -> Self::Output;

    fn segments(self, source: Segment, relation: SegmentRelation, target: Segment) -> Self::Output;
}

impl Cycle {
    pub fn closed<C: Closes>(self, closes: C) -> C::Output {
        match self {
            Cycle::Nodes { source, relation, target } => closes.nodes(source, relation, target),
            Cycle::Segments { source, relation, target } => closes.segments(source, relation, target),
        }
    }
}

pub trait Refuses: From<StoreFailure> {
    fn cyclic(cycle: Cycle) -> Self;
}

pub trait Acyclic {
    type End;

    const CLOSED: Closed;

    fn keyed(end: &Self::End, statements: &mut impl Execute) -> Result<Keyed, StoreFailure>;

    fn cycle(source: Self::End, target: Self::End) -> Cycle;
}

#[derive(Clone, Copy, Debug)]
pub struct Refines;

#[derive(Clone, Copy, Debug)]
pub struct Supersedes;

#[derive(Clone, Copy, Debug)]
pub struct SegmentSupersedes;

#[derive(Clone, Copy, Debug)]
pub struct SegmentCorrects;
