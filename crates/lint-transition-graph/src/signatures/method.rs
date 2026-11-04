mod owner;
mod taken;

use std::fmt;

use equivalence::{Equivalence, Erased, Keyed};
use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;
use transition_facts::Mutation;

use crate::edge::Edge;
use crate::listed::Listed;
use crate::workspace_types::WorkspaceTypes;
use owner::Owner;
use taken::Taken;

#[derive(Clone, Debug)]
pub struct Method<'system, 'found, 'file> {
    edge: &'system Edge<'found, 'file>,
    owner: Owner<'found, 'file>,
    equivalence: Equivalence,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature<'found, 'file> {
    owner: Owner<'found, 'file>,
    takes: Vec<Taken<'found>>,
    gives: Taken<'found>,
    mutation: Mutating,
    name: Option<&'found Name>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Mutating(Mutation);

pub(super) struct Located<'method, 'system, 'found, 'file>(&'method Method<'system, 'found, 'file>);

impl<'system, 'found, 'file> Method<'system, 'found, 'file> {
    pub(super) fn of_edge(
        types: &WorkspaceTypes<'found>,
        edge: &'system Edge<'found, 'file>,
        equivalence: Equivalence,
    ) -> Option<Method<'system, 'found, 'file>> {
        let transition = edge.transition();
        let krate: &'found CrateName = AsRef::<CrateName>::as_ref(transition.context());
        types.owner(transition).map(|end| Method {
            edge,
            owner: Owner::of_end(end, krate, edge.site().path()),
            equivalence,
        })
    }

    pub(super) fn owner(&self) -> &Owner<'found, 'file> {
        &self.owner
    }

    pub(super) fn located(&self) -> Located<'_, 'system, 'found, 'file> {
        Located(self)
    }
}

impl<'found, 'file> Keyed for Method<'_, 'found, 'file> {
    type Key = Signature<'found, 'file>;

    fn key(&self) -> Signature<'found, 'file> {
        let function = self.edge.transition().function();
        let ends = self.edge.ends();
        let takes = ends.takes().iter().zip(function.takings()).map(|(end, wrapping)| Taken::new(*end, *wrapping));
        Signature {
            owner: self.owner,
            takes: takes.collect(),
            gives: Taken::new(*AsRef::as_ref(ends), function.wrapping()),
            mutation: Mutating(function.mutation()),
            name: self.equivalence.erases(Erased::Names).map_or(Some(AsRef::<Name>::as_ref(function)), |_| None),
        }
    }
}

impl AsRef<Name> for Method<'_, '_, '_> {
    fn as_ref(&self) -> &Name {
        self.edge.transition().function().as_ref()
    }
}

impl<'file> AsRef<Site<'file>> for Method<'_, '_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.edge.transition().as_ref()
    }
}

impl fmt::Display for Located<'_, '_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}", AsRef::<Name>::as_ref(self.0), AsRef::<Site>::as_ref(self.0))
    }
}

impl fmt::Display for Mutating {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Mutation::Absent => Ok(()),
            Mutation::ThroughSelf => write!(formatter, ", mutating self"),
            Mutation::ThroughParameter => write!(formatter, ", mutating a parameter"),
        }
    }
}

impl fmt::Display for Signature<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let takes = Listed::<_, ','>::joined(self.takes.iter());
        write!(formatter, "({takes}) -> {}{}", self.gives, self.mutation)
    }
}
