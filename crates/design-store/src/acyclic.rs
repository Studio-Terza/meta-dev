mod boundary;

use design_model::{Inversion, Misstaged, NodeId, NodeRelation, Segment, SegmentRelation};
use named_derive::Named;
use store_ports::{Statements, StoreFailure};

use crate::Unpaid;

pub(crate) use boundary::{Closing, Keyed};

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Closed {
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

#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    #[error("the edge would close a cycle of its relation")]
    Cyclic(Cycle),
    #[error("the superseding node refines the node it would supersede")]
    Ancestral { old: NodeId, by: NodeId },
    #[error("{0}")]
    Inverted(#[from] Inversion),
    #[error("{0}")]
    Misstaged(#[from] Misstaged),
    #[error("{0}")]
    Unpaid(#[from] Unpaid),
    #[error("{0}")]
    Store(#[from] StoreFailure),
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

pub(crate) trait Acyclic {
    type End;

    const CLOSED: Closed;

    fn keyed(end: &Self::End, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure>;

    fn cycle(source: Self::End, target: Self::End) -> Cycle;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Refines;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Supersedes;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SegmentSupersedes;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SegmentCorrects;

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
