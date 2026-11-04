mod step;

use std::collections::{BTreeMap, BTreeSet};

use equivalence::{Equivalence, Move, Refinement};
use lint_core::Site;

use crate::algebra::Operation;
use crate::algebra_laws::attached::Attached;
use crate::count::Count;
use crate::edge::Edge;
use crate::end::Node;
use crate::system::TypeSystem;
use step::Step;

type Moves<'found, 'laws> = BTreeMap<Node<'found>, BTreeSet<Move<Node<'found>, Step<'found, 'laws>>>>;

#[derive(Debug)]
pub(super) struct Behaviour<'laws, 'found, 'file> {
    refinement: Refinement<Node<'found>, Step<'found, 'laws>>,
    sites: BTreeMap<Node<'found>, Site<'file>>,
}

impl<'laws, 'found, 'file> Behaviour<'laws, 'found, 'file> {
    pub(super) fn read(
        system: &TypeSystem<'found, 'file>,
        attached: &'laws Attached,
        equivalence: Equivalence,
    ) -> Behaviour<'laws, 'found, 'file> {
        let operations: BTreeMap<Count, &Operation<'found, 'file>> =
            system.algebras().operations().map(|operation| (operation.ordinal(), operation)).collect();
        let moves = system.edges().iter().enumerate().fold(Moves::new(), |moves, (ordinal, edge)| {
            let laws =
                operations.get(&Count::new(ordinal)).map_or(&[][..], |operation| attached.of_operation(operation));
            Step::read(edge, laws, equivalence).into_iter().fold(moves, |moves, step| Self::moved(moves, edge, step))
        });
        let sites = system
            .edges()
            .iter()
            .flat_map(|edge| edge.ends().taken().map(move |node| Placed { node: *node, site: edge.site() }))
            .fold(BTreeMap::new(), |mut sites: BTreeMap<Node<'found>, Site<'file>>, placed| {
                sites.entry(placed.node).and_modify(|site| *site = (*site).min(placed.site)).or_insert(placed.site);
                sites
            });
        Behaviour { refinement: Refinement::new(moves), sites }
    }

    pub(super) fn refinement(&self) -> &Refinement<Node<'found>, Step<'found, 'laws>> {
        &self.refinement
    }

    pub(super) fn site(&self, node: &Node<'found>) -> Option<Site<'file>> {
        self.sites.get(node).copied()
    }

    fn moved(moves: Moves<'found, 'laws>, edge: &Edge<'found, '_>, step: Step<'found, 'laws>) -> Moves<'found, 'laws> {
        let taken: BTreeSet<Node<'found>> = edge.ends().taken().copied().collect();
        taken.iter().fold(moves, |mut moves, node| {
            let with = taken.iter().copied().filter(|other| other != node).collect();
            moves.entry(*node).or_default().insert(Move::new(step, edge.ends().as_ref().node().copied(), with));
            moves
        })
    }
}

struct Placed<'found, 'file> {
    node: Node<'found>,
    site: Site<'file>,
}
