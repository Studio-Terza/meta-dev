mod step;

use std::collections::{BTreeMap, BTreeSet};

use equivalence::{Equivalence, Move, Refinement};
use graph::Placed;
use graph::Property;
use lint_core::Site;
use lint_layout::Name;

use crate::algebra::Operation;
use crate::algebra_laws::attached::Attached;
use crate::count::Count;
use crate::edge::Edge;
use crate::end::Node;
use crate::system::TypeSystem;
use crate::workspace_types::WorkspaceTypes;
use step::Step;

type Moves<'found> = BTreeMap<Node<'found>, Vec<Move<Node<'found>, Step<'found>, Property>>>;

#[derive(Debug)]
pub(super) struct Behaviour<'found, 'file> {
    refinement: Refinement<Node<'found>, Step<'found>, Property>,
    sites: BTreeMap<Node<'found>, Site<'file>>,
}

impl<'found, 'file> Behaviour<'found, 'file> {
    pub(super) fn read(
        system: &TypeSystem<'found, 'file>,
        attached: &Attached,
        equivalence: Equivalence,
    ) -> Behaviour<'found, 'file> {
        let operations: BTreeMap<Count, &Operation<'found, 'file>> =
            system.algebras().operations().map(|operation| (operation.ordinal(), operation)).collect();
        let moves = system.edges().iter().enumerate().fold(Moves::new(), |moves, (ordinal, edge)| {
            let laws =
                operations.get(&Count::new(ordinal)).map_or(&[][..], |operation| attached.of_operation(operation));
            Step::read(edge).into_iter().fold(moves, |so_far, step| Self::moved(so_far, edge, step, laws))
        });
        let sites = moves.keys().fold(BTreeMap::new(), |mut sites, node| {
            sites.extend(Self::declared(system, node).map(|site| (*node, site)));
            sites
        });
        Behaviour { refinement: Refinement::new(moves, equivalence.laws()), sites }
    }

    pub(super) fn refinement(&self) -> &Refinement<Node<'found>, Step<'found>, Property> {
        &self.refinement
    }

    pub(super) fn site(&self, node: &Node<'found>) -> Option<Site<'file>> {
        self.sites.get(node).copied()
    }

    fn declared(system: &TypeSystem<'found, 'file>, node: &Node<'found>) -> Option<Site<'file>> {
        let model = system.model();
        model
            .items()
            .filter(|placed| placed.krate() == node.krate())
            .find(|placed| {
                model
                    .declared(placed.entity().declared())
                    .filter(|item| WorkspaceTypes::TYPES.contains(&item.kind()))
                    .is_some_and(|item| AsRef::<Name>::as_ref(item) == AsRef::<Name>::as_ref(node))
            })
            .map(Placed::site)
    }

    fn moved(moves: Moves<'found>, edge: &Edge<'found, '_>, step: Step<'found>, laws: &[Property]) -> Moves<'found> {
        let taken: BTreeSet<Node<'found>> = edge.ends().taken().copied().collect();
        taken.iter().fold(moves, |mut gathered, node| {
            let with = taken.iter().copied().filter(|other| other != node).collect();
            let reached = edge.ends().as_ref().node().copied();
            gathered.entry(*node).or_default().push(Move::new(step, reached, with, laws.to_vec()));
            gathered
        })
    }
}
