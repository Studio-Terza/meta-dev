use std::fmt;

use equivalence::{Group, Holes, Placed};
use lint_core::{Fix, Site, Violation};
use newtype::Counter;

use super::behaviour::Behaviour;
use super::coloured::{Coloured, Spread};
use crate::count::Count;
use crate::end::Node;
use crate::listed::Listed;

#[derive(Debug)]
struct Member<'found, 'file> {
    node: Node<'found>,
    site: Site<'file>,
}

impl fmt::Display for Member<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}", self.node, self.site)
    }
}

#[derive(Debug)]
pub(super) struct Structure<'found, 'file> {
    members: Vec<Member<'found, 'file>>,
    moves: Count,
    holes: Holes,
}

impl<'found, 'file> Structure<'found, 'file> {
    const LEAST_MOVES: usize = 2;
    const LEAST_MEMBERS: usize = 2;

    pub(super) fn of_group(
        group: &Group<Placed<Node<'found>>>,
        behaviour: &Behaviour<'found, 'file>,
    ) -> Option<Structure<'found, 'file>> {
        let members: Vec<Member<'found, 'file>> = group
            .members()
            .iter()
            .map(Placed::node)
            .filter_map(|node| behaviour.site(&node).map(|site| Member { node, site }))
            .collect();
        let moves = group
            .members()
            .first()
            .map(Placed::node)
            .map_or(Count::zero(), |node| Count::new(behaviour.refinement().moves_of(&node).len()));
        Some(Structure { members, moves, holes: group.holes() })
            .filter(|structure| structure.moves.count() >= Self::LEAST_MOVES)
            .filter(|structure| structure.members.len() >= Self::LEAST_MEMBERS)
    }
}

impl<'file> From<&Structure<'_, 'file>> for Option<Violation<'file>> {
    fn from(structure: &Structure<'_, 'file>) -> Option<Violation<'file>> {
        structure.members.iter().map(|member| member.site).min().map(|site| Violation::new(site, structure))
    }
}

impl fmt::Display for Structure<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} types answer {} moves alike under one renaming: {}; one generic over {} type parameters covers them",
            Count::new(self.members.len()),
            self.moves,
            Listed::<_, ','>::joined(self.members.iter()),
            self.holes
        )
    }
}

impl lint_core::Finding for &Structure<'_, '_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "write the shared structure once, as one generic type or trait over the types the renaming swaps, and make each member an instance of it; where they mean different things, give them moves of their own",
        )
    }
}

#[derive(Debug)]
pub(super) struct Tally {
    spread: Spread,
    structures: Count,
    grouped: Count,
}

impl Tally {
    pub(super) fn counted(coloured: &Coloured<'_>, structures: &[Structure<'_, '_>]) -> Tally {
        Tally {
            spread: coloured.spread(),
            structures: Count::new(structures.len()),
            grouped: Count::total(structures.iter().map(|structure| Count::new(structure.members.len()))),
        }
    }

    pub(super) fn placed(self, first: Option<Site<'_>>) -> Option<Violation<'_>> {
        first.map(|site| Violation::new(site, self))
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workspace: {}; {} parallel structures hold {} types",
            self.spread, self.structures, self.grouped
        )
    }
}

impl lint_core::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("write each parallel structure once as one generic")
    }
}
