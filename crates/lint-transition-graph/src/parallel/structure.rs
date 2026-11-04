use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use equivalence::{Colour, Group, Holes, Renaming};
use lint_core::{Fix, Site, Violation};
use newtype::Counter;

use super::behaviour::Behaviour;
use super::coloured::Coloured;
use crate::count::Count;
use crate::end::Node;
use crate::listed::Listed;

#[derive(Debug)]
struct Member<'found, 'file> {
    node: Node<'found>,
    site: Site<'file>,
    renaming: Renaming<Node<'found>>,
}

impl<'found, 'file> Member<'found, 'file> {
    fn renamed(
        first: Node<'found>,
        node: Node<'found>,
        behaviour: &Behaviour<'_, 'found, 'file>,
        colours: &BTreeMap<Node<'found>, Colour>,
    ) -> Option<Member<'found, 'file>> {
        let renaming = behaviour.refinement().renaming(first, node, colours);
        renaming.zip(behaviour.site(&node)).map(|(renaming, site)| Member { node, site, renaming })
    }
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
}

impl<'found, 'file> Structure<'found, 'file> {
    const LEAST_MOVES: usize = 2;
    const LEAST_MEMBERS: usize = 2;

    pub(super) fn renamed(
        group: &Group<Coloured<'found>>,
        behaviour: &Behaviour<'_, 'found, 'file>,
        colours: &BTreeMap<Node<'found>, Colour>,
    ) -> Option<Structure<'found, 'file>> {
        let first = group.members().first().copied().map(Coloured::node);
        let moves = first
            .and_then(|node| behaviour.refinement().moves_of(&node))
            .map_or(Count::zero(), |found| Count::new(found.len()));
        let members: Vec<Member<'found, 'file>> = first
            .into_iter()
            .flat_map(|first| {
                group
                    .members()
                    .iter()
                    .copied()
                    .filter_map(move |member| Member::renamed(first, member.node(), behaviour, colours))
            })
            .collect();
        Some(Structure { members, moves })
            .filter(|structure| structure.moves.count() >= Self::LEAST_MOVES)
            .filter(|structure| structure.members.len() >= Self::LEAST_MEMBERS)
    }

    fn holes(&self) -> Holes {
        self.members.iter().map(|member| member.renaming.holes()).max().unwrap_or(Holes::zero())
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
            self.holes()
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
    types: Count,
    colours: Count,
    structures: Count,
    grouped: Count,
}

impl Tally {
    pub(super) fn counted(colours: &BTreeMap<Node<'_>, Colour>, structures: &[Structure<'_, '_>]) -> Tally {
        Tally {
            types: Count::new(colours.len()),
            colours: Count::new(colours.values().collect::<BTreeSet<&Colour>>().len()),
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
            "workspace: {} domain types with moves refine into {} colours; {} parallel structures hold {} types",
            self.types, self.colours, self.structures, self.grouped
        )
    }
}

impl lint_core::Finding for Tally {
    fn fix(&self) -> Fix {
        Fix::new("write each parallel structure once as one generic")
    }
}
