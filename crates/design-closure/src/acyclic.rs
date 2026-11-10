mod boundary;

use design_model::{NodeId, NodeRelation, Segment, SegmentRelation};
use named_derive::Named;
use store_ports::{Statements, StoreFailure};

pub use boundary::{Closing, Keyed};

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closed {
    #[name("refines")]
    Refines,
    #[name("supersedes")]
    Supersedes,
    #[name("segment-supersedes")]
    SegmentSupersedes,
    #[name("segment-corrects")]
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

    fn keyed(end: &Self::End, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure>;

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

impl Acyclic for Refines {
    type End = NodeId;

    const CLOSED: Closed = Closed::Refines;

    fn keyed(end: &NodeId, _statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        Ok(Keyed::node(*end))
    }

    fn cycle(source: NodeId, target: NodeId) -> Cycle {
        Cycle::Nodes { source, relation: NodeRelation::Refines, target }
    }
}

impl Acyclic for Supersedes {
    type End = NodeId;

    const CLOSED: Closed = Closed::Supersedes;

    fn keyed(end: &NodeId, _statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        Ok(Keyed::node(*end))
    }

    fn cycle(source: NodeId, target: NodeId) -> Cycle {
        Cycle::Nodes { source, relation: NodeRelation::Supersedes, target }
    }
}

impl Acyclic for SegmentSupersedes {
    type End = Segment;

    const CLOSED: Closed = Closed::SegmentSupersedes;

    fn keyed(end: &Segment, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        Keyed::segment(end, statements)
    }

    fn cycle(source: Segment, target: Segment) -> Cycle {
        Cycle::Segments { source, relation: SegmentRelation::Supersedes, target }
    }
}

impl Acyclic for SegmentCorrects {
    type End = Segment;

    const CLOSED: Closed = Closed::SegmentCorrects;

    fn keyed(end: &Segment, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        Keyed::segment(end, statements)
    }

    fn cycle(source: Segment, target: Segment) -> Cycle {
        Cycle::Segments { source, relation: SegmentRelation::Corrects, target }
    }
}
