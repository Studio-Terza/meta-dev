use std::fmt;

use algebra::{Monoid, Semigroup};
use graph::Mutation;
use lint_core::{Finding, Fix, Site, Violation};
use newtype::Counter;

use super::disguised::Disguised;
use crate::count::Count;
use crate::edge::Edge;
use crate::form::Acting;
use crate::transitions::{Declares, Ground};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shape {
    Unary,
    Builder,
    Binary,
    Mutating,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Standing {
    Undeclared,
    Fixed,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Shaped<'edge, 'file> {
    edge: &'edge Edge<'edge, 'file>,
    shape: Shape,
    standing: Standing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Shapes {
    unary: Count,
    builder: Count,
    binary: Count,
    mutating: Count,
    fixed: Count,
    disguised: Count,
}

#[derive(Debug)]
pub(super) struct Tallied<'file> {
    shapes: Shapes,
    first: Option<Site<'file>>,
}

impl<'edge, 'file> Shaped<'edge, 'file> {
    pub(super) fn of_edge(edge: &'edge Edge<'edge, 'file>) -> Option<Shaped<'edge, 'file>> {
        let transition = edge.transition();
        let core = (*AsRef::<Ground>::as_ref(transition) == Ground::Core).then_some(edge);
        let shape = core.and_then(|_| Shape::of_edge(edge))?;
        let declared = Some(shape)
            .filter(|found| *found == Shape::Binary)
            .filter(|_| transition.context().filler().declares() == Declares::Semigroup);
        let standing = edge.fixed_by().map_or(Standing::Undeclared, |_| Standing::Fixed);
        declared.map_or(Some(Shaped { edge, shape, standing }), |_| None)
    }
}

impl<'file> AsRef<Site<'file>> for Shaped<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.edge.transition().as_ref()
    }
}

impl<'file> From<&Shaped<'_, 'file>> for Option<Violation<'file>> {
    fn from(shaped: &Shaped<'_, 'file>) -> Option<Violation<'file>> {
        match shaped.standing {
            Standing::Undeclared => Some(Violation::new(*shaped.as_ref(), Undeclared(*shaped))),
            Standing::Fixed => None,
        }
    }
}

impl Shape {
    fn of_edge(edge: &Edge<'_, '_>) -> Option<Shape> {
        let function = edge.transition().function();
        let ground = *AsRef::<Ground>::as_ref(edge.transition());
        let mutation =
            function.held().operating(function.mutation(), ground).map_or(function.mutation(), |()| Mutation::Absent);
        Shape::of_mutation(mutation).or_else(|| edge.form().algebraic().and_then(Shape::of_acting))
    }

    fn of_mutation(mutation: Mutation) -> Option<Shape> {
        match mutation {
            Mutation::ThroughSelf | Mutation::ThroughParameter => Some(Shape::Mutating),
            Mutation::Absent => None,
        }
    }

    fn of_acting(acting: &Acting<'_>) -> Option<Shape> {
        let held = acting.held().nonzero()?;
        let single = (held == Count::new(1)).then_some(Shape::Unary);
        Some(acting.pure().map_or(Shape::Builder, |_| single.unwrap_or(Shape::Binary)))
    }

    fn fix(self) -> Fix {
        match self {
            Shape::Unary => Fix::new(
                "write the refinement type this produces (transition-graph D14), so the edge goes from the type to a new type",
            ),
            Shape::Builder => Fix::new(
                "make each fixed step a typestate edge T<S1> to T<S2>, or turn a per-item accumulation into contributions merged by a declared algebra::Monoid",
            ),
            Shape::Binary => Fix::new("implement algebra::Semigroup for the type and make this its combine"),
            Shape::Mutating => Fix::new("return a new value, or move the mutation into a store operation"),
        }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Unary => write!(formatter, "unary T -> T"),
            Shape::Builder => write!(formatter, "builder (T, X) -> T"),
            Shape::Binary => write!(formatter, "binary (T, T) -> T"),
            Shape::Mutating => write!(formatter, "mutating &mut"),
        }
    }
}

#[derive(Debug)]
struct Undeclared<'edge, 'file>(Shaped<'edge, 'file>);

impl fmt::Display for Undeclared<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} is an undeclared {}", self.0.edge.transition(), self.0.shape)
    }
}

impl Finding for Undeclared<'_, '_> {
    fn fix(&self) -> Fix {
        self.0.shape.fix()
    }
}

impl Shapes {
    pub(super) fn tallied<'file>(shaped: &[Shaped<'_, 'file>], disguised: &[Disguised<'file>]) -> Tallied<'file> {
        let edges = shaped.iter().map(Shapes::of_shaped);
        let impls = disguised.iter().map(|_| Shapes { disguised: Count::new(1), ..Shapes::default() });
        let sites = shaped.iter().map(|found| *found.as_ref()).chain(disguised.iter().map(Site::from));
        Tallied { shapes: Monoid::concat(edges.chain(impls)), first: sites.min() }
    }

    fn of_shaped(shaped: &Shaped<'_, '_>) -> Shapes {
        match shaped.standing {
            Standing::Fixed => Shapes { fixed: Count::new(1), ..Shapes::default() },
            Standing::Undeclared => Shapes::of_shape(shaped.shape),
        }
    }

    fn of_shape(shape: Shape) -> Shapes {
        match shape {
            Shape::Unary => Shapes { unary: Count::new(1), ..Shapes::default() },
            Shape::Builder => Shapes { builder: Count::new(1), ..Shapes::default() },
            Shape::Binary => Shapes { binary: Count::new(1), ..Shapes::default() },
            Shape::Mutating => Shapes { mutating: Count::new(1), ..Shapes::default() },
        }
    }

    fn undeclared(self) -> Count {
        self.unary.plus(self.builder).plus(self.binary).plus(self.mutating)
    }
}

impl Semigroup for Shapes {
    fn combine(self, other: Shapes) -> Shapes {
        Shapes {
            unary: self.unary.plus(other.unary),
            builder: self.builder.plus(other.builder),
            binary: self.binary.plus(other.binary),
            mutating: self.mutating.plus(other.mutating),
            fixed: self.fixed.plus(other.fixed),
            disguised: self.disguised.plus(other.disguised),
        }
    }
}

impl Monoid for Shapes {}

impl<'file> From<Tallied<'file>> for Option<Violation<'file>> {
    fn from(tallied: Tallied<'file>) -> Option<Violation<'file>> {
        tallied.first.map(|site| Violation::new(site, Workspace(tallied.shapes)))
    }
}

#[derive(Debug)]
struct Workspace(Shapes);

impl fmt::Display for Workspace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let shapes = self.0;
        write!(
            formatter,
            "workspace: {} undeclared same-type edges, {} unary, {} builder, {} binary, {} mutating; {} std impls are an undeclared algebra; {} edges with a signature fixed by a foreign trait are counted apart",
            shapes.undeclared(),
            shapes.unary,
            shapes.builder,
            shapes.binary,
            shapes.mutating,
            shapes.disguised,
            shapes.fixed
        )
    }
}

impl Finding for Workspace {
    fn fix(&self) -> Fix {
        Fix::new(
            "remove each site with the fix its own finding names; the lint blocks once the workspace count reaches zero",
        )
    }
}
