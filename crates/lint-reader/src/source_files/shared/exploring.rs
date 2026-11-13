use std::collections::{BTreeMap, BTreeSet};
use std::mem;
use std::path::PathBuf;

use file_ports::Files;
use lint_roots::{Fact, Survey};

use super::assembly::Assembly;
use super::boundary::Partition;
use super::boundary::{Change, Flights};
use super::declared::{Declared, Explored};
use crate::source_files::Stage;
use crate::source_files::located::{Located, Node};

#[derive(Debug)]
pub(super) struct Queued<'tree> {
    node: Node<'tree>,
    owner: Option<Partition>,
}

impl<'tree> Queued<'tree> {
    pub(super) fn node(&self) -> &Node<'tree> {
        &self.node
    }

    fn nested(&self, declared: &Declared, survey: &Survey, share: Partition) -> Vec<Queued<'tree>> {
        let owned = self.owner.map(|_| share);
        declared
            .nested(|child, kind| Queued { node: self.node.child_of(child, survey), owner: kind.owner(owned, share) })
    }

    fn claimable(&self, share: Partition) -> Option<&Queued<'tree>> {
        Some(self).filter(|queued| queued.owner.is_none_or(|owner| owner == share))
    }
}

#[derive(Debug, Default)]
pub(super) struct Exploring<'tree> {
    survey: Survey,
    queue: Vec<Queued<'tree>>,
    visited: BTreeSet<Located<'tree>>,
    flights: Flights,
    explored: BTreeMap<Located<'tree>, Explored>,
}

impl<'tree> Exploring<'tree> {
    pub(super) fn started(starts: Vec<Node<'tree>>) -> Exploring<'tree> {
        let mut visited = BTreeSet::new();
        let queue = starts
            .into_iter()
            .filter(|node| visited.insert(node.located().clone()))
            .map(|node| Queued { node, owner: None })
            .collect();
        Exploring { queue, visited, ..Exploring::default() }
    }

    pub(super) fn waiting(&self, share: Partition) -> Option<Flights> {
        let claimable = self.queue.iter().find_map(|queued| queued.claimable(share));
        let pending = self.queue.first().map(|_| self.flights);
        claimable.map_or(Some(self.flights), |_| None).and_then(|flights| flights.airborne().or(pending))
    }

    pub(super) fn claimed(&mut self, share: Partition) -> Option<Queued<'tree>> {
        let position = self.queue.iter().position(|queued| queued.claimable(share).is_some())?;
        self.flights = self.flights.flown(Change::Lifted);
        Some(self.queue.remove(position))
    }

    pub(super) fn learned(&mut self, files: &impl Files, candidates: &[PathBuf]) -> &Survey {
        self.survey = mem::take(&mut self.survey).learn(files, Fact::Followed, candidates);
        &self.survey
    }

    pub(super) fn settled(&mut self, queued: &Queued<'tree>, explored: Explored, share: Partition) {
        let next = explored.as_ref().map_or_default(|declared| queued.nested(declared, &self.survey, share));
        let fresh: Vec<Queued<'tree>> =
            next.into_iter().filter(|found| self.visited.insert(found.node.located().clone())).collect();
        self.queue.extend(fresh);
        self.explored.insert(queued.node.located().clone(), explored);
        self.flights = self.flights.flown(Change::Landed);
    }

    pub(super) fn assembled(&mut self, stage: &'tree Stage<'tree>) -> Assembly<'tree> {
        Assembly::walked(stage, mem::take(&mut self.survey), mem::take(&mut self.explored))
    }
}
